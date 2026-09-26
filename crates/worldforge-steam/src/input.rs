use serde::{Deserialize, Serialize};

/// Controller glyph type for HUD button prompts.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ControllerGlyphFamily {
    SteamDeck,
    Xbox,
    PlayStation,
    NintendoSwitch,
    GenericGamepad,
}

/// Action mapping description in Steam Input.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControllerAction {
    pub name: String,
    pub description: String,
    pub action_type: String, // "Button", "AnalogStick", "Trigger"
    pub default_binding: String,
}

/// Steam Deck and Controller Profile Manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamInputProfile {
    pub is_steam_deck: bool,
    pub glyph_family: ControllerGlyphFamily,
    pub actions: Vec<ControllerAction>,
    pub deadzone: f32,
    pub sensitivity: f32,
    pub gyro_enabled: bool,
    pub radial_wheel_open: bool,
}

impl Default for SteamInputProfile {
    fn default() -> Self {
        Self {
            is_steam_deck: false,
            glyph_family: ControllerGlyphFamily::SteamDeck,
            actions: vec![
                ControllerAction {
                    name: "CameraPan".into(),
                    description: "Pan battlefield tactical camera".into(),
                    action_type: "AnalogStick".into(),
                    default_binding: "LeftStick".into(),
                },
                ControllerAction {
                    name: "CameraOrbit".into(),
                    description: "Rotate 3D pitch and azimuth".into(),
                    action_type: "AnalogStick".into(),
                    default_binding: "RightStick".into(),
                },
                ControllerAction {
                    name: "RadialCommandWheel".into(),
                    description: "Hold to open 15-suite radial menu".into(),
                    action_type: "Button".into(),
                    default_binding: "LeftBumper".into(),
                },
                ControllerAction {
                    name: "TacticalAbility1".into(),
                    description: "Fire Orbital Kinetic Strike".into(),
                    action_type: "Trigger".into(),
                    default_binding: "RightTrigger".into(),
                },
                ControllerAction {
                    name: "TacticalAbility2".into(),
                    description: "Deploy Aegis Energy Dome".into(),
                    action_type: "Trigger".into(),
                    default_binding: "LeftTrigger".into(),
                },
                ControllerAction {
                    name: "InteractSelect".into(),
                    description: "Select unit or confirm dialog".into(),
                    action_type: "Button".into(),
                    default_binding: "ButtonA".into(),
                },
                ControllerAction {
                    name: "CancelClose".into(),
                    description: "Cancel order or close dialog".into(),
                    action_type: "Button".into(),
                    default_binding: "ButtonB".into(),
                },
                ControllerAction {
                    name: "BulletTimeToggle".into(),
                    description: "Toggle slow-motion tactical analysis".into(),
                    action_type: "Button".into(),
                    default_binding: "ButtonX".into(),
                },
                ControllerAction {
                    name: "CycleCameraView".into(),
                    description: "Cycle between Orbit, City, Battle, and Chase cam".into(),
                    action_type: "Button".into(),
                    default_binding: "ButtonY".into(),
                },
                ControllerAction {
                    name: "PauseToggle".into(),
                    description: "Pause / Resume time simulation".into(),
                    action_type: "Button".into(),
                    default_binding: "MenuStart".into(),
                },
            ],
            deadzone: 0.12,
            sensitivity: 1.0,
            gyro_enabled: true,
            radial_wheel_open: false,
        }
    }
}

impl SteamInputProfile {
    pub fn new_steam_deck() -> Self {
        Self {
            is_steam_deck: true,
            glyph_family: ControllerGlyphFamily::SteamDeck,
            gyro_enabled: true,
            ..Default::default()
        }
    }

    /// Generates the official Steam Input Action Set VDF manifest (`game_actions_480.vdf`).
    pub fn generate_steam_input_vdf(&self) -> String {
        r##""ActionManifest"
{
    "configurations"
    {
        "controller_neptune"
        {
            "0"
            {
                "path" "controller_configuration_deck.vdf"
            }
        }
        "controller_xboxone"
        {
            "0"
            {
                "path" "controller_configuration_deck.vdf"
            }
        }
    }
    "actions"
    {
        "InGameControls"
        {
            "title" "#Set_InGameControls"
            "StickPadGyro"
            {
                "CameraPan"
                {
                    "title" "#Action_CameraPan"
                    "input_mode" "joystick_move"
                }
                "CameraOrbit"
                {
                    "title" "#Action_CameraOrbit"
                    "input_mode" "joystick_camera"
                }
            }
            "AnalogTrigger"
            {
                "TacticalAbility1"
                {
                    "title" "#Action_TacticalAbility1"
                    "action_name" "TacticalAbility1"
                }
                "TacticalAbility2"
                {
                    "title" "#Action_TacticalAbility2"
                    "action_name" "TacticalAbility2"
                }
            }
            "Button"
            {
                "RadialCommandWheel"
                {
                    "title" "#Action_RadialCommandWheel"
                }
                "InteractSelect"
                {
                    "title" "#Action_InteractSelect"
                }
                "CancelClose"
                {
                    "title" "#Action_CancelClose"
                }
                "BulletTimeToggle"
                {
                    "title" "#Action_BulletTimeToggle"
                }
                "CycleCameraView"
                {
                    "title" "#Action_CycleCameraView"
                }
                "PauseToggle"
                {
                    "title" "#Action_PauseToggle"
                }
            }
        }
    }
}"##.to_string()
    }
}
