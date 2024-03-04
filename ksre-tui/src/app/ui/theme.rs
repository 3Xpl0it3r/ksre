use ratatui::style::Color;

pub trait Kanagawa {
    const SUMLINK0: Color = Color::from_u32(0x0016161D);
    const SUMLINK1: Color = Color::from_u32(0x001F1F28);
    const BLUEWINTER: Color = Color::from_u32(0x00252535);
    const RED_SAMURAI: Color = Color::from_u32(0x00E82424);
    const VIOLET_SPRING1: Color = Color::from_u32(0x00938AA9);
    const VIOLET_ONI: Color = Color::from_u32(0x00957FB8);
    const BLUE_CRYSTAL: Color = Color::from_u32(0x007E9CD8);
    const VIOLET_SPRING: Color = Color::from_u32(0x009CABCA);
    const BLUE_SPRING: Color = Color::from_u32(0x007FB4CA);
    const BLUE_LIGHT: Color = Color::from_u32(0x00A3D4D5);
    const AQUA_WAVE2: Color = Color::from_u32(0x007AA89F);
    const GREEN_SPRING: Color = Color::from_u32(0x0098BB6C);
    const YELLOW_BOAT: Color = Color::from_u32(0x00C0A36E);
    const YELLOW_CARP: Color = Color::from_u32(0x00E6C384);
    const PINK_SAKURA: Color = Color::from_u32(0x00D27E99);
    const RED_PEACH: Color = Color::from_u32(0x00FF5D62);
    const RED_WAVE: Color = Color::from_u32(0x00E46876);
    const ORANGE_SURIMI: Color = Color::from_u32(0x00FFA066);

}

pub struct DefaultTheme {}

impl Kanagawa for DefaultTheme {}
