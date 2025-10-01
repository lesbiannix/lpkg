#[derive(Default)]
pub struct Document {
    width: u32,
    height: u32,
    view_box: Option<String>,
    role: Option<String>,
    aria_label: Option<(String, String)>,
    title: Option<String>,
    desc: Option<String>,
    defs: Vec<String>,
    elements: Vec<String>,
}

impl Document {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    pub fn view_box(mut self, value: &str) -> Self {
        self.view_box = Some(value.to_string());
        self
    }

    pub fn role(mut self, value: &str) -> Self {
        self.role = Some(value.to_string());
        self
    }

    pub fn aria_label(mut self, title_id: &str, desc_id: &str) -> Self {
        self.aria_label = Some((title_id.to_string(), desc_id.to_string()));
        self
    }

    pub fn title(mut self, value: &str) -> Self {
        self.title = Some(value.to_string());
        self
    }

    pub fn desc(mut self, value: &str) -> Self {
        self.desc = Some(value.to_string());
        self
    }

    pub fn add_defs(mut self, defs: Defs) -> Self {
        self.defs.push(defs.finish());
        self
    }

    pub fn add_element(mut self, element: impl Into<String>) -> Self {
        self.elements.push(element.into());
        self
    }

    pub fn finish(self) -> String {
        let Document {
            width,
            height,
            view_box,
            role,
            aria_label,
            title,
            desc,
            defs,
            elements,
        } = self;

        let mut out = String::new();
        out.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\"",
            width, height
        ));
        if let Some(view_box) = view_box {
            out.push_str(&format!(" viewBox=\"{}\"", view_box));
        }
        if let Some(role) = role {
            out.push_str(&format!(" role=\"{}\"", role));
        }
        let (title_id, desc_id) = aria_label
            .as_ref()
            .map(|ids| (ids.0.as_str(), ids.1.as_str()))
            .unwrap_or(("title", "desc"));
        if aria_label.is_some() {
            out.push_str(&format!(" aria-labelledby=\"{} {}\"", title_id, desc_id));
        }
        out.push_str(">");
        out.push('\n');

        if let Some(title) = title {
            out.push_str(&format!("  <title id=\"{}\">{}</title>\n", title_id, title));
        }
        if let Some(desc) = desc {
            out.push_str(&format!("  <desc id=\"{}\">{}</desc>\n", desc_id, desc));
        }

        if !defs.is_empty() {
            out.push_str("  <defs>\n");
            for block in &defs {
                out.push_str(block);
            }
            out.push_str("  </defs>\n");
        }

        for element in &elements {
            out.push_str(element);
            out.push('\n');
        }

        out.push_str("</svg>\n");
        out
    }
}

pub struct Defs {
    content: Vec<String>,
}

impl Defs {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }

    pub fn linear_gradient(mut self, id: &str, gradient: Gradient) -> Self {
        self.content
            .push(format!("    {}\n", gradient.render_linear(id)));
        self
    }

    pub fn radial_gradient(mut self, id: &str, gradient: RadialGradient) -> Self {
        self.content.push(format!("    {}\n", gradient.render(id)));
        self
    }

    pub fn filter(mut self, id: &str, filter: Filter) -> Self {
        self.content.push(format!("    {}\n", filter.render(id)));
        self
    }

    pub fn finish(self) -> String {
        self.content.concat()
    }
}

pub struct Gradient {
    x1: String,
    y1: String,
    x2: String,
    y2: String,
    stops: Vec<String>,
}

impl Gradient {
    pub fn new(x1: &str, y1: &str, x2: &str, y2: &str) -> Self {
        Self {
            x1: x1.to_string(),
            y1: y1.to_string(),
            x2: x2.to_string(),
            y2: y2.to_string(),
            stops: Vec::new(),
        }
    }

    pub fn stop(mut self, offset: &str, attrs: &[(&str, &str)]) -> Self {
        let mut tag = format!("<stop offset=\"{}\"", offset);
        for (k, v) in attrs {
            tag.push_str(&format!(" {}=\"{}\"", k, v));
        }
        tag.push_str(" />");
        self.stops.push(tag);
        self
    }

    fn render_linear(&self, id: &str) -> String {
        let mut out = format!(
            "<linearGradient id=\"{}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\">\n",
            id, self.x1, self.y1, self.x2, self.y2
        );
        for stop in &self.stops {
            out.push_str("      ");
            out.push_str(stop);
            out.push('\n');
        }
        out.push_str("    </linearGradient>");
        out
    }
}

pub struct RadialGradient {
    cx: String,
    cy: String,
    r: String,
    stops: Vec<String>,
}

impl RadialGradient {
    pub fn new(cx: &str, cy: &str, r: &str) -> Self {
        Self {
            cx: cx.to_string(),
            cy: cy.to_string(),
            r: r.to_string(),
            stops: Vec::new(),
        }
    }

    pub fn stop(mut self, offset: &str, attrs: &[(&str, &str)]) -> Self {
        let mut tag = format!("<stop offset=\"{}\"", offset);
        for (k, v) in attrs {
            tag.push_str(&format!(" {}=\"{}\"", k, v));
        }
        tag.push_str(" />");
        self.stops.push(tag);
        self
    }

    fn render(&self, id: &str) -> String {
        let mut out = format!(
            "<radialGradient id=\"{}\" cx=\"{}\" cy=\"{}\" r=\"{}\">\n",
            id, self.cx, self.cy, self.r
        );
        for stop in &self.stops {
            out.push_str("      ");
            out.push_str(stop);
            out.push('\n');
        }
        out.push_str("    </radialGradient>");
        out
    }
}

pub struct Filter {
    attrs: Vec<(String, String)>,
    content: Vec<String>,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            attrs: Vec::new(),
            content: Vec::new(),
        }
    }

    pub fn attr(mut self, key: &str, value: &str) -> Self {
        self.attrs.push((key.to_string(), value.to_string()));
        self
    }

    pub fn raw(mut self, markup: &str) -> Self {
        self.content.push(format!("      {}\n", markup));
        self
    }

    fn render(&self, id: &str) -> String {
        let attrs = self
            .attrs
            .iter()
            .map(|(k, v)| format!(" {}=\"{}\"", k, v))
            .collect::<String>();
        let mut out = format!("<filter id=\"{}\"{}>\n", id, attrs);
        for child in &self.content {
            out.push_str(child);
        }
        out.push_str("    </filter>");
        out
    }
}

pub struct Element {
    tag: String,
    attrs: Vec<(String, String)>,
    content: Option<String>,
}

impl Element {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
            attrs: Vec::new(),
            content: None,
        }
    }

    pub fn attr(mut self, key: &str, value: &str) -> Self {
        self.attrs.push((key.to_string(), value.to_string()));
        self
    }

    pub fn text(mut self, text: &str) -> String {
        self.content = Some(text.to_string());
        self.render()
    }

    pub fn empty(mut self) -> String {
        self.content = None;
        self.render()
    }

    fn render(&self) -> String {
        let attrs = self
            .attrs
            .iter()
            .map(|(k, v)| format!(" {}=\"{}\"", k, v))
            .collect::<String>();
        if let Some(content) = &self.content {
            format!(
                "  <{tag}{attrs}>{content}</{tag}>",
                tag = self.tag,
                attrs = attrs,
                content = content
            )
        } else {
            format!("  <{tag}{attrs} />", tag = self.tag, attrs = attrs)
        }
    }
}

pub struct Group {
    attrs: Vec<(String, String)>,
    children: Vec<String>,
}

impl Group {
    pub fn new() -> Self {
        Self {
            attrs: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn attr(mut self, key: &str, value: &str) -> Self {
        self.attrs.push((key.to_string(), value.to_string()));
        self
    }

    pub fn child(mut self, element: impl Into<String>) -> Self {
        self.children.push(element.into());
        self
    }

    pub fn render(&self) -> String {
        let attrs = self
            .attrs
            .iter()
            .map(|(k, v)| format!(" {}=\"{}\"", k, v))
            .collect::<String>();
        let mut out = format!("  <g{}>\n", attrs);
        for child in &self.children {
            out.push_str(child);
            out.push('\n');
        }
        out.push_str("  </g>");
        out
    }
}

impl From<Group> for String {
    fn from(group: Group) -> Self {
        group.render()
    }
}

impl From<Element> for String {
    fn from(element: Element) -> Self {
        element.render()
    }
}

pub fn path(d: &str) -> String {
    Element::new("path").attr("d", d).empty()
}
