use std::{collections::HashMap, num::NonZeroU64, sync::{Arc, Mutex}};

use songbird::{CoreEvent, EventContext, EventHandler};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use vesper::{macros::command, prelude::{async_trait, DefaultCommandResult, SlashContext}};

use crate::Context;

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

                        tokio::spawn(async move{

                            // 2秒後に同じ人がしゃべっていたら、それを結合する
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            let mut wav_by_user = wav_by_user.lock().unwrap();

                            // removeしたはずなのに、2秒後に存在しているということは、2秒後の現在、続けて同じ人がしゃべっているということ。
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

                            // 2秒後に同じ人がしゃべっていなかった場合、Whisperに音声データを渡す
                            println!("{}: {}s", user_id, wav.len() / (2 * 48000));

                            // 奇数番目だけ採用し、wavをモノラルに変換
                            let wav_mono: Vec<i16> = wav.into_iter().enumerate().filter_map(|(i, x)| if i % 2 == 0 { Some(x) } else { None }).collect();

                            // 音声認識
                            let recognized_text = speech_to_text(&wav_mono).unwrap();

                            println!("{}: {}", user_id, recognized_text);

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
}

impl ReceiverContext {
    fn new() -> Self {
        Self {
            wav_by_user: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

fn speech_to_text(wav_48khz_1ch: &[i16]) -> anyhow::Result<String> {

    Ok("".to_string())
}