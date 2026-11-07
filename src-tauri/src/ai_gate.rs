//! پورت ۱:۱ از `ai/AiGate.kt` — چرا هوش مصنوعی همین حالا در دسترس است یا نه.

use crate::engine::{exit_socks_port, CHAIN_SOCKS_PORT, LOCAL_SOCKS_PORT};
use crate::profile::{ConnectionProfile, TransportBackend};
use crate::state::ConnectionState;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AiGate {
    /// آماده: کلید هست، تونل بالاست.
    Ready,
    /// هنوز کلید API وارد نشده.
    NoKey,
    /// مدلی انتخاب نشده و برای این کلید هم مدلی کشف نشده.
    NoModel,
    /// تونل پایین است (یا هنوز بالا نیامده)، پس گوگل دست‌نیافتنی است.
    Disconnected,
    /// متصل، ولی روی حالت نامناسب.
    WrongMode,
}

impl AiGate {
    pub fn ready(self) -> bool {
        matches!(self, AiGate::Ready)
    }

    pub fn code(self) -> &'static str {
        match self {
            AiGate::Ready => "READY",
            AiGate::NoKey => "NO_KEY",
            AiGate::NoModel => "NO_MODEL",
            AiGate::Disconnected => "DISCONNECTED",
            AiGate::WrongMode => "WRONG_MODE",
        }
    }
}

/// وضعیت دروازه را حل می‌کند.
pub fn evaluate(
    state: ConnectionState,
    backend: TransportBackend,
    has_key: bool,
    has_model: bool,
) -> AiGate {
    if !has_key {
        return AiGate::NoKey;
    }
    if state != ConnectionState::Connected {
        return AiGate::Disconnected;
    }
    let route = crate::custom::ai_policy::active();
    let eligible = route
        .map(|r| r.mode.map(crate::custom::ai_policy::custom_eligible).unwrap_or(true))
        .unwrap_or(true);
    if !eligible {
        return AiGate::WrongMode;
    }
    if !has_model {
        return AiGate::NoModel;
    }
    AiGate::Ready
}

/// پورت SOCKS5 محلی که ترافیک هوش مصنوعی را حمل می‌کند.
pub fn socks_port(backend: TransportBackend) -> u16 {
    if backend.is_chained() {
        CHAIN_SOCKS_PORT
    } else {
        let p = exit_socks_port();
        if p != 0 { p } else { LOCAL_SOCKS_PORT }
    }
}

/// راحتیِ فراخوان‌ها: هر دو مقدارِ لازم از یک پروفایل.
pub fn socks_port_for(profile: &ConnectionProfile) -> Result<u16, String> {
    if let Some(route) = crate::custom::ai_policy::active() {
        if route.port != 0 {
            return Ok(route.port);
        }
    }
    let exit_port = exit_socks_port();
    if exit_port != 0 {
        Ok(exit_port)
    } else if profile.backend.is_chained() {
        Ok(CHAIN_SOCKS_PORT)
    } else {
        Ok(LOCAL_SOCKS_PORT)
    }
}
