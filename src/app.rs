use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use nostr_sdk::prelude::*;
use ratatui::prelude::Rect;
use tokio::sync::mpsc;

use crate::{
    action::Action,
    components::{Component, FpsCounter, Home, StatusBar},
    config::Config,
    mode::Mode,
    nostr::Connection,
    nostr::ConnectionProcess,
    tui,
};

use nostr_sdk::types::time::SystemTime;
use nostr_sdk::types::time::UNIX_EPOCH;
use std::error::Error;

pub struct App {
    pub config: Config,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub components: Vec<Box<dyn Component>>,
    pub should_quit: bool,
    pub should_suspend: bool,
    pub mode: Mode,
    pub last_tick_key_events: Vec<KeyEvent>,
}

impl App {
    const CUSTOM_PORT: usize = 8000;
    fn prepend<T>(v: Vec<T>, s: &[T]) -> Vec<T>
    where
        T: Clone,
    {
        let mut tmp: Vec<_> = s.to_owned();
        tmp.extend(v);
        tmp
    }
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let home = Home::new();
        let fps = FpsCounter::default();
        let config = Config::new()?;
        let pubkey = Keys::from_sk_str(config.privatekey.as_str())?.public_key();
        let status_bar = StatusBar::new(pubkey, None, None, true);
        let mode = Mode::Home;
        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![Box::new(home), Box::new(fps), Box::new(status_bar)],
            should_quit: false,
            should_suspend: false,
            config,
            mode,
            last_tick_key_events: Vec::new(),
        })
    }

    async fn blockhash() -> Result<String, Box<dyn Error>> {
        let blockhash = reqwest::get("https://mempool.space/api/blocks/tip/hash")
            .await?
            .text()
            .await?;

        log::info!("blockhash() = {blockhash:?}");
        Ok(blockhash)
    }
    async fn blockheight() -> Result<String, Box<dyn Error>> {
        let blockheight = reqwest::get("https://mempool.space/api/blocks/tip/height")
            .await?
            .text()
            .await?;

        log::info!("blockheight() = {blockheight:?}");
        Ok(blockheight)
    }
    async fn nanos() -> Result<u32, Box<dyn Error>> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos();
        log::info!("nanos():{nanos}");
        Ok(nanos)
    }

    async fn do_stuff_async() -> Result<(), Box<dyn Error>> {
        // async work
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos();
        log::info!("do_stuff_async:{nanos}");
        Ok(())
    }

    async fn more_async_work() -> Result<(), Box<dyn Error>> {
        // more here
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos();
        log::info!("more_async_work:{nanos}");
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        let self_nanos = Self::nanos().await.unwrap();
        log::info!("self_nanos:{self_nanos:?}");

        let (action_tx, mut action_rx) = mpsc::unbounded_channel();

        let mut tui = tui::Tui::new()?
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        // tui.mouse(true);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(action_tx.clone())?;
        }

        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }

        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        let keys = Keys::from_sk_str(&self.config.privatekey.clone())?;
        let conn = Connection::new(keys.clone(), self.config.relays.clone()).await?;
        let (mut req_rx, event_tx, terminate_tx, conn_wrapper) = ConnectionProcess::new(conn)?;
        conn_wrapper.run();

        loop {
            let loop_nanos = Self::nanos().await.unwrap();
            log::info!("loop_nanos:{loop_nanos:?}");
            if let Some(e) = tui.next().await {
                match e {
                    tui::Event::Quit => action_tx.send(Action::Quit)?,
                    tui::Event::Tick => action_tx.send(Action::Tick)?,
                    tui::Event::Render => action_tx.send(Action::Render)?,
                    tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                    tui::Event::Key(key) => {
                        action_tx.send(Action::Key(key))?;

                        if let Some(keymap) = self.config.keybindings.get(&self.mode) {
                            if let Some(action) = keymap.get(&vec![key]) {
                                log::info!("Got action: {action:?}");
                                action_tx.send(action.clone())?;
                            } else {
                                // If the key was not handled as a single key action,
                                // then consider it for multi-key combinations.
                                self.last_tick_key_events.push(key);

                                // Check for multi-key combinations
                                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                                    log::info!("Got action: {action:?}");
                                    action_tx.send(action.clone())?;
                                }
                            }
                        };
                    }
                    _ => {}
                }
                for component in self.components.iter_mut() {
                    if let Some(action) = component.handle_events(Some(e.clone()))? {
                        action_tx.send(action)?;
                    }
                }
            }

            while let Ok(event) = req_rx.try_recv() {
                action_tx.send(Action::ReceiveEvent(event))?;
            }

            while let Ok(action) = action_rx.try_recv() {
                if action != Action::Tick && action != Action::Render {
                    log::debug!("{action:?}");
                }
                match action {
                    Action::Tick => {
                        self.last_tick_key_events.drain(..);
                        let action_tick_blockheight =
                            Self::blockheight().await.unwrap_or(String::from("0"));
                        log::info!("Action::Tick_blockheight:{action_tick_blockheight:?}");
                        let action_tick_blockhash =
                            Self::blockhash().await.unwrap_or(String::from("0"));
                        log::info!("Action::Tick_blockhash:{action_tick_blockhash:?}");

                        //
                    }
                    Action::Quit => self.should_quit = true,
                    Action::Suspend => self.should_suspend = true,
                    Action::Resume => self.should_suspend = false,
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))?;
                        tui.draw(|f| {
                            for component in self.components.iter_mut() {
                                let r = component.draw(f, f.size());
                                if let Err(e) = r {
                                    action_tx
                                        .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                        .unwrap();
                                }
                            }
                        })?;
                    }
                    Action::Render => {
                        tui.draw(|f| {
                            for component in self.components.iter_mut() {
                                let r = component.draw(f, f.size());
                                if let Err(e) = r {
                                    action_tx
                                        .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                        .unwrap();
                                }
                            }
                        })?;
                    }
                    Action::ReceiveEvent(ref event) => {
                        log::info!("Got nostr event: {event:?}");
                    }
                    Action::SendReaction((id, pubkey)) => {
                        let event = EventBuilder::new_reaction(id, pubkey, "+").to_event(&keys)?;
                        log::info!("Send reaction: {event:?}");
                        event_tx.send(event)?;
                        let note1 = id.to_bech32()?;
                        action_tx.send(Action::SystemMessage(format!("[Liked] {note1}")))?;
                    }
                    Action::SendRepost((id, pubkey)) => {
                        let event = EventBuilder::repost(id, pubkey).to_event(&keys)?;
                        log::info!("Send repost: {event:?}");
                        event_tx.send(event)?;
                        let note1 = id.to_bech32()?;
                        action_tx.send(Action::SystemMessage(format!("[Reposted] {note1}")))?;
                    }

                    Action::SendTextNote(ref content, ref tags) => {
                        let event = EventBuilder::new_text_note(content, tags.iter().cloned())
                            .to_event(&keys)?;
                        log::info!("Send text note: {event:?}");
                        event_tx.send(event)?;
                        action_tx.send(Action::SystemMessage(format!("[Posted] {content}")))?;
                    }

                    _ => {}
                }
                for component in self.components.iter_mut() {
                    if let Some(action) = component.update(action.clone())? {
                        action_tx.send(action)?
                    };
                }
            }
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                tui = tui::Tui::new()?
                    .tick_rate(self.tick_rate)
                    .frame_rate(self.frame_rate);
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                terminate_tx.send(())?;
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }
}
