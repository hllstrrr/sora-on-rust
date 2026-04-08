pub mod cmd;
pub mod downloader {
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
    pub mod group;
    pub mod kick;
    pub mod promote;
}
pub mod root {
    pub mod exec;
    pub mod set;
}