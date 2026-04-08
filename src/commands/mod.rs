pub mod cmd;
pub mod downloader {
    pub mod reels;
    pub mod spotify;
}
pub mod general {
    pub mod info;
    pub mod menu;
    pub mod ping;
}
pub mod group {
    pub mod add;
    pub mod demote;
    pub mod gc;
    pub mod kick;
    pub mod promote;
}
pub mod root {
    pub mod exec;
    pub mod set;
}
pub mod tools {
    pub mod debug;
}