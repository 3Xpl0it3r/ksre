use ratatui::style::Color;

 pub trait Kanagawa {
    const Sumlink0: Color = Color::from_u32(0x0016161D);
    const Sumlink1: Color = Color::from_u32(0x001F1F28);
    const BlueWinter: Color = Color::from_u32(0x00252535);
    const RedSamurai: Color = Color::from_u32(0x00E82424);
    const VioletSpring1: Color = Color::from_u32(0x00938AA9);
    const VioletOni: Color = Color::from_u32(0x00957FB8);
    const BlueCrystal: Color = Color::from_u32(0x007E9CD8);
    const VioletSpring: Color = Color::from_u32(0x009CABCA);
    const BlueSpring: Color = Color::from_u32(0x007FB4CA);
    const BlueLight: Color = Color::from_u32(0x00A3D4D5);
    const AquaWave2: Color = Color::from_u32(0x007AA89F);
    const GreenSpring: Color = Color::from_u32(0x0098BB6C);
    const YellowBoat: Color = Color::from_u32(0x00C0A36E);
    const YellowCarp: Color = Color::from_u32(0x00E6C384);
    const PinkSakura: Color = Color::from_u32(0x00D27E99);
    const RedPeach: Color = Color::from_u32(0x00FF5D62);
    const RedWave: Color = Color::from_u32(0x00E46876);
    const OrangeSurimi: Color = Color::from_u32(0x00FFA066);
}

pub struct DefaultTheme {}

impl Kanagawa for DefaultTheme {}
