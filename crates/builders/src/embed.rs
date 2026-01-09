use discord_rs_model::embed::{Embed, EmbedFooter, EmbedImage, EmbedThumbnail, EmbedAuthor, EmbedField};

#[derive(Debug, Clone, Default)]
pub struct EmbedBuilder {
    inner: Embed,
}

impl EmbedBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.inner.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.inner.description = Some(description.into());
        self
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.inner.url = Some(url.into());
        self
    }

    pub fn timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.inner.timestamp = Some(timestamp.into());
        self
    }

    pub fn color(mut self, color: u32) -> Self {
        self.inner.color = Some(color);
        self
    }

    pub fn footer(mut self, text: impl Into<String>, icon_url: Option<String>) -> Self {
        self.inner.footer = Some(EmbedFooter {
            text: text.into(),
            icon_url,
            proxy_icon_url: None,
        });
        self
    }

    pub fn image(mut self, url: impl Into<String>) -> Self {
        self.inner.image = Some(EmbedImage {
            url: url.into(),
            proxy_url: None,
            height: None,
            width: None,
        });
        self
    }

    pub fn thumbnail(mut self, url: impl Into<String>) -> Self {
        self.inner.thumbnail = Some(EmbedThumbnail {
            url: url.into(),
            proxy_url: None,
            height: None,
            width: None,
        });
        self
    }

    pub fn author(mut self, name: impl Into<String>, url: Option<String>, icon_url: Option<String>) -> Self {
        self.inner.author = Some(EmbedAuthor {
            name: name.into(),
            url,
            icon_url,
            proxy_icon_url: None,
        });
        self
    }

    pub fn add_field(mut self, name: impl Into<String>, value: impl Into<String>, inline: bool) -> Self {
        self.inner.fields.push(EmbedField {
            name: name.into(),
            value: value.into(),
            inline,
        });
        self
    }

    pub fn build(self) -> Embed {
        self.inner
    }
}

impl From<EmbedBuilder> for Embed {
    fn from(builder: EmbedBuilder) -> Self {
        builder.build()
    }
}
