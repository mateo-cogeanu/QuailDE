use crate::apps::AppCategory;

/// ShellTheme centralizes QuailDE's visual language so the shell can move away
/// from scattered color literals and toward one cohesive, tweakable theme.
#[derive(Debug, Clone, Copy)]
pub struct ShellTheme {
    pub wallpaper_top: u32,
    pub wallpaper_bottom: u32,
    pub wallpaper_glow_a: u32,
    pub wallpaper_glow_b: u32,
    pub wallpaper_glow_c: u32,
    pub surface_shadow: u32,
    pub surface_bg: u32,
    pub surface_alt_bg: u32,
    pub surface_border: u32,
    pub panel_bg: u32,
    pub panel_border: u32,
    pub panel_button: u32,
    pub panel_button_active: u32,
    pub launcher_bg: u32,
    pub launcher_header: u32,
    pub launcher_search: u32,
    pub launcher_sidebar_selected: u32,
    pub launcher_sidebar_icon: u32,
    pub launcher_tile: u32,
    pub launcher_tile_selected: u32,
    pub launcher_tile_outline: u32,
    pub terminal_bg: u32,
    pub terminal_header: u32,
    pub terminal_header_focused: u32,
    pub terminal_content: u32,
    pub terminal_caret: u32,
    pub window_bg: u32,
    pub window_bg_focused: u32,
    pub window_header: u32,
    pub window_header_focused: u32,
    pub overlay_bg: u32,
    pub overlay_card: u32,
    pub overlay_card_alt: u32,
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_muted: u32,
    pub text_warning: u32,
}

/// shell_theme returns QuailDE's current dark shell theme. Keeping it in one
/// place makes broad visual reworks possible without touching every painter.
pub fn shell_theme() -> ShellTheme {
    ShellTheme {
        wallpaper_top: 0xFF0A1016,
        wallpaper_bottom: 0xFF131D2B,
        wallpaper_glow_a: 0x2244C7A2,
        wallpaper_glow_b: 0x1D628CFF,
        wallpaper_glow_c: 0x1A8D62FF,
        surface_shadow: 0x3A05070B,
        surface_bg: 0xF1141820,
        surface_alt_bg: 0xFF181F29,
        surface_border: 0xFF2C3644,
        panel_bg: 0xEE0E141C,
        panel_border: 0xFF293241,
        panel_button: 0xFF1B2430,
        panel_button_active: 0xFF234564,
        launcher_bg: 0xEF141B24,
        launcher_header: 0xF019212C,
        launcher_search: 0xFF202935,
        launcher_sidebar_selected: 0xFF1F4460,
        launcher_sidebar_icon: 0xFF5B88B8,
        launcher_tile: 0xFF151C26,
        launcher_tile_selected: 0xFF203B55,
        launcher_tile_outline: 0x16FF_FFFF,
        terminal_bg: 0xF20E1319,
        terminal_header: 0xFF151D26,
        terminal_header_focused: 0xFF1A2734,
        terminal_content: 0xFF090D12,
        terminal_caret: 0xFF87E36A,
        window_bg: 0xFF141A22,
        window_bg_focused: 0xFF101720,
        window_header: 0xFF1A232E,
        window_header_focused: 0xFF203041,
        overlay_bg: 0x4010141D,
        overlay_card: 0xF0141A22,
        overlay_card_alt: 0xFF1A2330,
        text_primary: 0xFFDCE6F1,
        text_secondary: 0xFFB5C4D3,
        text_muted: 0x88A1B1C2,
        text_warning: 0xFFF0D4D9,
    }
}

/// accent_for_category keeps app tiles and dock icons visually consistent.
pub fn accent_for_category(category: AppCategory) -> u32 {
    match category {
        AppCategory::Terminal => 0xFF57B7FF,
        AppCategory::Browser => 0xFFFFB357,
        AppCategory::Files => 0xFF54C991,
        AppCategory::Editor => 0xFFC486FF,
        AppCategory::Utility => 0xFF8EA6C0,
    }
}
