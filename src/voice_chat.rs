use std::{
    collections::HashMap,
    io::Cursor,
    num::NonZeroU64,
    sync::{Arc, Mutex},
};

use anyhow::{bail, Context as AnyhowContext};
use hound::WavReader;
use reqwest::{header, multipart::Form};
use songbird::{id::GuildId, CoreEvent, EventContext, EventHandler};
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use vesper::{
    macros::command,
    prelude::{async_trait, DefaultCommandResult, SlashContext},
};

use crate::Context;

#[command]
#[description = "ボイスチャンネルに招待する"]
pub async fn join(ctx: &mut SlashContext<Arc<Context>>) -> DefaultCommandResult {
    let guild_id = ctx
        .interaction
        .guild_id
        .ok_or_else(|| anyhow::anyhow!("Guild ID not found"))?;

    let channel_id: NonZeroU64 = std::env::var("VOICE_CHANNEL_ID")
        .context("Expected a voice channel ID in the environment")?
        .parse()
        .context("Voice channel ID is not a valid number")?;

    ctx.interaction_client
        .create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some("おじゃまします！".to_string()),
                    ..Default::default()
                }),
            },
        )
        .await?;

    let songbird = ctx.data.songbird.clone();

    tokio::spawn(async move {
        let call = match songbird.join(guild_id, channel_id).await {
            Ok(call) => call,
            Err(why) => {
                tracing::error!("Failed to join voice channel: {:?}", why);
                return;
            }
        };

        let mut handler = call.lock().await;
        let receiver = Receiver::new(songbird);

        handler.remove_all_global_events();
        handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), receiver.clone());
        handler.add_global_event(CoreEvent::VoiceTick.into(), receiver.clone());

        tracing::info!("Successfully joined voice channel");
    });

    Ok(())
}

#[command]
#[description = "ボイスチャンネルから追い出す"]
pub async fn leave(ctx: &mut SlashContext<Arc<Context>>) -> DefaultCommandResult {
    let guild_id = ctx
        .interaction
        .guild_id
        .ok_or_else(|| anyhow::anyhow!("Guild ID not found"))?;

    // イベントハンドラーを先に削除
    if let Some(handler) = ctx.data.songbird.get(guild_id) {
        let mut handler = handler.lock().await;
        handler.remove_all_global_events();
    }

    // その後でボイスチャンネルから退出
    ctx.data.songbird.leave(guild_id).await?;

    ctx.interaction_client
        .create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some("おじゃましました！".to_string()),
                    ..Default::default()
                }),
            },
        )
        .await?;

    Ok(())
}

#[derive(Clone)]
struct Receiver {
    inner: Arc<ReceiverContext>,
}

impl Receiver {
    fn new(songbird: Arc<songbird::Songbird>) -> Self {
        tracing::info!("Receiver spawned!");
        Self {
            inner: Arc::new(ReceiverContext::new(songbird)),
        }
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        tracing::info!("Receiver dropped!");
    }
}

#[async_trait]
impl EventHandler for Receiver {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<songbird::Event> {
        let receiver_context = self.inner.clone();

        match ctx {
            EventContext::SpeakingStateUpdate(speaking) => {
                tracing::debug!("Speaking state update: {:?}", speaking);
            }
            EventContext::VoiceTick(tick) => {
                let voice_hashmap = &tick.speaking;

                for (user_id, speaking) in voice_hashmap.iter() {
                    if let Some(wav_20ms_48khz_2ch_lrlr) = &speaking.decoded_voice {
                        let mut wav_by_user = receiver_context.wav_by_user.lock().unwrap();
                        wav_by_user
                            .entry(*user_id)
                            .or_default()
                            .extend(wav_20ms_48khz_2ch_lrlr);
                    }
                }

                // しゃべっていないユーザーに対して
                for user_id in tick.silent.iter() {
                    let wav = {
                        let mut wav_by_user = receiver_context.wav_by_user.lock().unwrap();

                        if !wav_by_user.contains_key(user_id) {
                            continue;
                        }

                        // 音声データを取り出して削除
                        wav_by_user.remove(user_id)
                    };

                    let Some(wav) = wav else { continue };

                    // tokioに渡す
                    let wav_by_user = receiver_context.wav_by_user.clone();
                    let user_id = *user_id;
                    let assistant_history = receiver_context.assistant_history.clone();
                    let songbird = receiver_context.songbird.clone();
                    let is_speaking = receiver_context.is_speaking.clone();

                    tokio::spawn(async move {
                        // 指定秒後に同じ人がしゃべっていたら、それを結合する
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                        let wav = {
                            let mut wav_by_user = wav_by_user.lock().unwrap();

                            // removeしたはずなのに、指定秒後に存在しているということは、指定秒後の現在、続けて同じ人がしゃべっているということ。
                            if let Some(wav_now_recording) = wav_by_user.get(&user_id) {
                                let mut combined_wav = wav.clone();
                                combined_wav.extend(wav_now_recording);

                                // 録音中の音声データに上書き
                                wav_by_user.insert(user_id, combined_wav);

                                // それ以降の処理は中断
                                return;
                            }

                            wav
                        };

                        // 1秒未満の音声データは無視
                        if wav.len() < 48000 * 2 {
                            return;
                        }

                        // アシスタントがしゃべっている最中は、音声データを受け付けない
                        {
                            let mut s = is_speaking.lock().unwrap();
                            if *s {
                                tracing::debug!("Assistant is speaking, ignoring user audio");
                                return;
                            }
                            *s = true;
                        }

                        // 奇数番目だけ採用し、wavをモノラルに変換
                        let wav_mono: Vec<i16> = wav
                            .into_iter()
                            .enumerate()
                            .filter_map(|(i, x)| if i % 2 == 0 { Some(x) } else { None })
                            .collect();

                        // 音声認識と応答生成
                        let result = process_audio(
                            wav_mono,
                            user_id,
                            assistant_history,
                            songbird,
                            is_speaking.clone(),
                        )
                        .await;

                        if let Err(e) = result {
                            tracing::error!("Failed to process audio: {:?}", e);
                            let mut s = is_speaking.lock().unwrap();
                            *s = false;
                        }
                    });
                }
            }
            _ => {}
        }

        None
    }
}

async fn process_audio(
    wav_mono: Vec<i16>,
    user_id: u32,
    assistant_history: Arc<Mutex<crate::llm::History>>,
    songbird: Arc<songbird::Songbird>,
    is_speaking: Arc<Mutex<bool>>,
) -> anyhow::Result<()> {
    // 音声認識
    let recognized_text = speech_to_text(&wav_mono).await?;
    tracing::info!("{}: {}", user_id, recognized_text);

    let voice_chat_model = std::env::var("VOICE_CHAT_MODEL")
        .context("Expected a voice chat model in the environment")?;

    // LLMに問い合わせ
    let response = {
        let mut assistant_history = assistant_history.lock().unwrap();
        assistant_history.push_as_user(&recognized_text);
        assistant_history.clone()
    }
    .get_with_system("かよわい女の子のような口調で返信してください。女の子の名前はミーシェです。女の子はご主人様と電話しています。電話だから、返答も短めにね。")
    .request(&voice_chat_model)
    .await?;

    {
        let mut assistant_history = assistant_history.lock().unwrap();
        assistant_history.push_as_assistant(&response);
    }

    // 音声合成
    let response_wav = text_to_speech(&response).await?;

    let guild_id: NonZeroU64 = std::env::var("GUILD_ID")
        .context("Expected a guild ID in the environment")?
        .parse()
        .context("Guild ID is not a valid number")?;

    let guild_id = GuildId::from(guild_id);
    let call = songbird
        .get(guild_id)
        .ok_or_else(|| anyhow::anyhow!("Not connected to voice channel"))?;

    let secs = {
        let mut call = call.lock().await;
        let secs = get_wav_duration_secs(&response_wav);
        let audio = response_wav.into();
        call.play_input(audio);
        secs
    };

    tracing::info!("Playing audio for {:.2} seconds", secs);
    tokio::time::sleep(std::time::Duration::from_secs_f64(secs)).await;
    tracing::info!("Finished playing audio");

    {
        let mut s = is_speaking.lock().unwrap();
        *s = false;
        tracing::info!("Assistant is not speaking anymore");
    }

    Ok(())
}

struct ReceiverContext {
    wav_by_user: Arc<Mutex<HashMap<u32, Vec<i16>>>>,
    assistant_history: Arc<Mutex<crate::llm::History>>,
    songbird: Arc<songbird::Songbird>,
    is_speaking: Arc<Mutex<bool>>,
}

impl ReceiverContext {
    fn new(songbird: Arc<songbird::Songbird>) -> Self {
        Self {
            wav_by_user: Arc::new(Mutex::new(HashMap::new())),
            assistant_history: Arc::new(Mutex::new(crate::llm::History::new())),
            songbird,
            is_speaking: Arc::new(Mutex::new(false)),
        }
    }
}

async fn speech_to_text(wav_48khz_1ch: &[i16]) -> anyhow::Result<String> {
    let client = reqwest::Client::new();

    let wav_48khz_1ch = normalize_audio(wav_48khz_1ch);
    let wavdata = make_wav_file(&wav_48khz_1ch)?;

    let multipart = Form::new()
        .text("model", "whisper-1")
        .text("response_format", "text")
        .part(
            "file",
            reqwest::multipart::Part::bytes(wavdata).file_name("audio.wav"),
        );

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {}", std::env::var("OPENAI_API_KEY")?)
            .parse()
            .context("Invalid authorization header")?,
    );

    let response = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .headers(headers)
        .multipart(multipart)
        .send()
        .await
        .context("Failed to send request to OpenAI")?;

    let response_in_text = response
        .text()
        .await
        .context("Failed to read response text")?;

    Ok(response_in_text)
}

fn make_wav_file(wav_48khz_1ch: &[i16]) -> anyhow::Result<Vec<u8>> {
    use hound::{WavSpec, WavWriter};

    let spec = WavSpec {
        channels: 1,
        sample_rate: 48000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buffer = Vec::new();

    {
        let mut writer = WavWriter::new(Cursor::new(&mut buffer), spec)?;

        for &sample in wav_48khz_1ch {
            writer.write_sample(sample)?;
        }

        writer.finalize()?;
    }

    Ok(buffer)
}

async fn text_to_speech(text: &str) -> anyhow::Result<Vec<u8>> {
    let client = reqwest::ClientBuilder::new()
        .connect_timeout(std::time::Duration::from_millis(500))
        .build()?;

    let urls =
        std::env::var("COEIRO_API_URLS").context("Expected COEIRO_API_URLS in the environment")?;

    for url in urls.lines() {
        let is_ok = client.get(url).send().await.is_ok();

        if !is_ok {
            continue;
        }

        let response = client
            .post(format!("{}v1/predict", url))
            .json(&serde_json::json!({
                "speakerUuid": "292ea286-3d5f-f1cc-157c-66462a6a9d08",
                "styleId": 42,
                "text": text,
                "speedScale": 1.2,
                "volumeScale": 1.0,
                "prosodyDetail": [],
                "pitchScale": 0.0,
                "intonationScale": 1.2,
                "prePhonemeLength": 0.1,
                "postPhonemeLength": 0.5,
                "outputSamplingRate": 24000,
            }))
            .header("Content-Type", "application/json")
            .send()
            .await?
            .bytes()
            .await?;

        return Ok(response.to_vec());
    }

    bail!("All Coeiro API URLs are down")
}

fn get_wav_duration_secs(wav_data: &[u8]) -> f64 {
    let cursor = Cursor::new(wav_data);
    let reader = WavReader::new(cursor).ok().unwrap();
    let spec = reader.spec();
    let duration = reader.duration();

    // 秒数を計算 (サンプル数 / サンプルレート)
    duration as f64 / spec.sample_rate as f64
}

fn normalize_audio(samples: &[i16]) -> Vec<i16> {
    // i16::MIN の abs() はオーバーフローするので、f32 にキャストしてから abs() を取る
    let max_sample = samples
        .iter()
        .map(|&s| (s as f32).abs())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(1.0);

    if max_sample == 0.0 {
        return samples.to_vec();
    }

    let factor = 32767.0 / max_sample;
    samples
        .iter()
        .map(|&s| {
            let normalized = (s as f32 * factor).round();
            // クランプして i16 の範囲内に収める
            normalized.clamp(-32768.0, 32767.0) as i16
        })
        .collect()
}
