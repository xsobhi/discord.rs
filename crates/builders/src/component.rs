use discord_rs_model::component::{Component, Button, ButtonStyle, SelectMenu, SelectMenuOption, ActionRow};
use discord_rs_model::emoji::Emoji;

#[derive(Debug, Clone, Default)]
pub struct ActionRowBuilder {
    components: Vec<Component>,
}

impl ActionRowBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_button(mut self, button: ButtonBuilder) -> Self {
        self.components.push(Component::Button(button.build()));
        self
    }

    pub fn add_select_menu(mut self, menu: SelectMenuBuilder) -> Self {
        // We need to know the type, for now default to StringSelect (3) mapping in Component enum
        // Actually the Component enum variants map to types.
        // Let's assume StringSelect for this builder or improve type handling.
        // For simplicity in this phase, we map to StringSelect.
        self.components.push(Component::StringSelect(menu.build()));
        self
    }

    pub fn build(self) -> ActionRow {
        ActionRow {
            components: self.components,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ButtonBuilder {
    inner: Button,
}

impl ButtonBuilder {
    pub fn new(style: ButtonStyle, custom_id_or_url: impl Into<String>) -> Self {
        let val = custom_id_or_url.into();
        let (custom_id, url) = if style == ButtonStyle::Link {
            (None, Some(val))
        } else {
            (Some(val), None)
        };

        Self {
            inner: Button {
                style: Some(style),
                label: None,
                emoji: None,
                custom_id,
                url,
                disabled: false,
            }
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.inner.label = Some(label.into());
        self
    }

    pub fn emoji(mut self, emoji: Emoji) -> Self {
        self.inner.emoji = Some(emoji);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.inner.disabled = disabled;
        self
    }

    pub fn build(self) -> Button {
        self.inner
    }
}

#[derive(Debug, Clone, Default)]
pub struct SelectMenuBuilder {
    inner: SelectMenu,
}

impl SelectMenuBuilder {
    pub fn new(custom_id: impl Into<String>) -> Self {
        Self {
            inner: SelectMenu {
                custom_id: custom_id.into(),
                options: Some(Vec::new()),
                channel_types: None,
                placeholder: None,
                min_values: None,
                max_values: None,
                disabled: false,
            }
        }
    }

    pub fn add_option(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        let options = self.inner.options.get_or_insert_with(Vec::new);
        options.push(SelectMenuOption {
            label: label.into(),
            value: value.into(),
            description: None,
            emoji: None,
            default: false,
        });
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.inner.placeholder = Some(placeholder.into());
        self
    }

    pub fn min_values(mut self, min: u32) -> Self {
        self.inner.min_values = Some(min);
        self
    }

    pub fn max_values(mut self, max: u32) -> Self {
        self.inner.max_values = Some(max);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.inner.disabled = disabled;
        self
    }

    pub fn build(self) -> SelectMenu {
        self.inner
    }
}

impl From<ActionRowBuilder> for Component {
    fn from(builder: ActionRowBuilder) -> Self {
        Component::ActionRow(builder.build())
    }
}
