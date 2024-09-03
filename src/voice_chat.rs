use std::{collections::HashMap, io::Cursor, num::NonZeroU64, sync::{Arc, Mutex}};

use reqwest::{header, multipart::Form};
use songbird::{CoreEvent, EventContext, EventHandler};
use twilight_http::response;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use vesper::{macros::command, prelude::{async_trait, DefaultCommandResult, SlashContext}};

use crate::{assistant, Context};

#[command]
#[description = "ボイスチャンネルに招待する"]
pub async fn join(ctx: &mut SlashContext<Arc<Context>>) -> DefaultCommandResult {

    let guild_id = ctx.interaction.guild_id.unwrap();
    let channel_id: NonZeroU64 = std::env::var("VOICE_CHANNEL_ID").expect("Expected a voice channel ID in the environment").parse().expect("Voice channel ID is not a number");
    
    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some("おじゃまします！".to_string()),
                ..Default::default()
            })
        }
    ).await?;

    let songbird = ctx.data.songbird.clone();

    tokio::spawn(async move {
        let call = {
            match songbird.join(guild_id, channel_id).await {
                Ok(call) => call,
                Err(why) => {
                    println!("Failed to join a channel: {:?}", why);
                    songbird.get(guild_id).unwrap()
                }
            }
        };
        let mut handler = call.lock().await;
    
        let receiver = Receiver::new();

        handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), receiver.clone());
        handler.add_global_event(CoreEvent::VoiceTick.into(), receiver.clone());
    });

    Ok(())
}

#[command]
#[description = "ボイスチャンネルから追い出す"]
pub async fn leave(ctx: &mut SlashContext<Arc<Context>>) -> DefaultCommandResult {

    let guild_id = ctx.interaction.guild_id.unwrap();
    ctx.data.songbird.leave(guild_id).await?;

    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some("おじゃましました！".to_string()),
                ..Default::default()
            })
        }
    ).await?;

    Ok(())
}

#[derive(Clone)]
struct Receiver{
    inner: Arc<ReceiverContext>
}

impl Receiver {
    fn new() -> Self {
        Self {
            inner: Arc::new(ReceiverContext::new())
        }
    }
}

#[async_trait]
impl EventHandler for Receiver {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<songbird::Event>{

        let receiver_context = self.inner.clone();

        match ctx {
            EventContext::SpeakingStateUpdate(speaking) => {
                println!("{:?}", speaking);
            },
            EventContext::VoiceTick(tick) => {
                let voice_hashmap = &tick.speaking;

                for (user_id, speaking) in voice_hashmap.iter() {
                    if let Some(wav_20ms_48khz_2ch_lrlr) = &speaking.decoded_voice {
                        let mut wav_by_user = receiver_context.wav_by_user.lock().unwrap();
                        wav_by_user.entry(*user_id).or_default().extend(wav_20ms_48khz_2ch_lrlr);
                    }
                }


                // しゃべっていないユーザーに対して
                for user_id in tick.silent.iter() {
                    
                    let mut wav_by_user = receiver_context.wav_by_user.lock().unwrap();
                    
                    if wav_by_user.contains_key(user_id) {

                        // もし現在しゃべっていないユーザーの音声データがあれば、それを取り出す

                        let wav = wav_by_user.get(user_id).unwrap().clone();
                        wav_by_user.remove(user_id);

                        // tokioに渡す前にwav_by_userのロックを解放
                        std::mem::drop(wav_by_user);

                        // tokioに渡す
                        let wav_by_user = receiver_context.wav_by_user.clone();
                        let user_id = *user_id;
                        let assistant_history = receiver_context.assistant_history.clone();

                        tokio::spawn(async move{

                            // 指定秒後に同じ人がしゃべっていたら、それを結合する
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            let mut wav_by_user = wav_by_user.lock().unwrap();

                            // removeしたはずなのに、指定秒後に存在しているということは、指定秒後の現在、続けて同じ人がしゃべっているということ。
                            if wav_by_user.contains_key(&user_id) {
                                
                                let wav_now_recording = wav_by_user.get(&user_id).unwrap();
                                let wav_already_recorded = wav;

                                // 2つの音声データを結合
                                let mut wav = vec![];
                                wav.extend(wav_already_recorded);
                                wav.extend(wav_now_recording);

                                // 録音中の音声データに上書き
                                wav_by_user.insert(user_id, wav);

                                // それ以降の処理は中断
                                return;

                            }

                            // 指定秒後に同じ人がしゃべっていなかった場合、Whisperに音声データを渡す
                            println!("{}: {}s", user_id, wav.len() / (2 * 48000));

                            // 奇数番目だけ採用し、wavをモノラルに変換
                            let wav_mono: Vec<i16> = wav.into_iter().enumerate().filter_map(|(i, x)| if i % 2 == 0 { Some(x) } else { None }).collect();

                            // 音声認識
                            tokio::spawn(async move {
                                let recognized_text = speech_to_text(&wav_mono).await.unwrap();
                                
                                println!("{}: {}", user_id, recognized_text);

                                let response = {
                                        let mut assistant_history = assistant_history.lock().unwrap();
                                        assistant_history.push_as_user(&recognized_text);
                                        assistant_history.clone()
                                    }
                                    .get_with_system("かよわい女の子のような口調で返信してください。女の子の名前はミーシェです。女の子はご主人様と電話しています。")
                                    .request("gpt-4o").await.unwrap();

                                {
                                    let mut assistant_history = assistant_history.lock().unwrap();
                                    assistant_history.push_as_assistant(&response);
                                }

                                let response_wav = text_to_speech(&response).await.unwrap();

                            });
                            

                        });
                    }
                }
            }
            _ => {}
        }

        None
    }
}


struct ReceiverContext {
    wav_by_user: Arc<Mutex<HashMap<u32, Vec<i16>>>>,
    assistant_history: Arc<Mutex<crate::llm::History>>,
}

impl ReceiverContext {
    fn new() -> Self {
        Self {
            wav_by_user: Arc::new(Mutex::new(HashMap::new())),
            assistant_history: Arc::new(Mutex::new(crate::llm::History::new())),
        }
    }
}

async fn speech_to_text(wav_48khz_1ch: &[i16]) -> anyhow::Result<String> {

    let client = reqwest::Client::new();

    let wavdata = make_wav_file(wav_48khz_1ch)?;

    let multipart = Form::new()
        .text("model", "whisper-1")
        .text("response_format", "text")
        .part("file", reqwest::multipart::Part::bytes(wavdata).file_name("audio.wav"));

    let mut headers = header::HeaderMap::new();
    headers.insert("Authorization", ["Bearer ", &std::env::var("OPENAI_API_KEY")?].concat().parse()?);

    let response = client.post("https://api.openai.com/v1/audio/transcriptions")
        .headers(headers)
        .multipart(multipart)
        .send()
        .await?;

    let response_in_text = response.text().await?;

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

        writer.finalize()?; // WAVファイルをクローズして書き込みを完了する
    }

    Ok(buffer)
}

async fn text_to_speech(text: &str) -> anyhow::Result<Vec<u8>> {

    let client = reqwest::Client::new();

    let response = client.post("http://127.0.0.1:50032/v1/synthesis")
        .json(&serde_json::json!({
            "speakerUuid": "3c37646f-3881-5374-2a83-149267990abc",
            "styleId": 0,
            "text": text,
            "speedScale": 1.0,
            "volumeScale": 1.0,
            "prosodyDetail": [],
            "pitchScale": 0.0,
            "intonationScale": 1.0,
            "prePhonemeLength": 0.1,
            "postPhonemeLength": 0.5,
            "outputSamplingRate": 24000,
        }))
        .header("Content-Type", "application/json")
        .send()
        .await?
        .bytes()
        .await?;

    Ok(response.to_vec())
}