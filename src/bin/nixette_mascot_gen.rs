use anyhow::Result;
use package_management::svg_builder::{Defs, Document, Element, Gradient, Group};
use std::fs;

fn main() -> Result<()> {
    let svg = build_mascot_svg();
    fs::create_dir_all("assets")?;
    fs::write("assets/nixette-mascot.svg", svg)?;
    Ok(())
}

fn build_mascot_svg() -> String {
    let defs = Defs::new()
        .linear_gradient(
            "bgGrad",
            Gradient::new("0", "0", "0", "1")
                .stop("0%", &[("stop-color", "#312E81")])
                .stop("100%", &[("stop-color", "#1E1B4B")]),
        )
        .linear_gradient(
            "hairLeft",
            Gradient::new("0", "0", "1", "1")
                .stop("0%", &[("stop-color", "#55CDFC")])
                .stop("100%", &[("stop-color", "#0EA5E9")]),
        )
        .linear_gradient(
            "hairRight",
            Gradient::new("1", "0", "0", "1")
                .stop("0%", &[("stop-color", "#F7A8B8")])
                .stop("100%", &[("stop-color", "#FB7185")]),
        )
        .linear_gradient(
            "bellyGrad",
            Gradient::new("0", "0", "0", "1")
                .stop("0%", &[("stop-color", "#FFFFFF")])
                .stop("100%", &[("stop-color", "#E2E8F0")]),
        );

    let body = Group::new()
        .attr("transform", "translate(240 220)")
        .child(
            Element::new("path")
                .attr("d", "M-160 -20 C-140 -160 140 -160 160 -20 C180 140 60 220 0 220 C-60 220 -180 140 -160 -20")
                .attr("fill", "#0F172A")
                .empty(),
        )
        .child(
            Element::new("ellipse")
                .attr("cx", "0")
                .attr("cy", "40")
                .attr("rx", "120")
                .attr("ry", "140")
                .attr("fill", "#1E293B")
                .empty(),
        )
        .child(
            Element::new("path")
                .attr("d", "M-88 -80 Q-40 -140 0 -120 Q40 -140 88 -80")
                .attr("fill", "#1E293B")
                .empty(),
        )
        .child(
            Element::new("path")
                .attr("d", "M-96 -84 Q-60 -160 -8 -132 L-8 -40 Z")
                .attr("fill", "url(#hairLeft)")
                .empty(),
        )
        .child(
            Element::new("path")
                .attr("d", "M96 -84 Q60 -160 8 -132 L8 -40 Z")
                .attr("fill", "url(#hairRight)")
                .empty(),
        )
        .child(ellipse(-44.0, -8.0, 26.0, 32.0, "#FFFFFF"))
        .child(ellipse(44.0, -8.0, 26.0, 32.0, "#FFFFFF"))
        .child(circle(-44.0, -4.0, 14.0, "#0F172A"))
        .child(circle(44.0, -4.0, 14.0, "#0F172A"))
        .child(circle_with_opacity(-40.0, -8.0, 6.0, "#FFFFFF", 0.7))
        .child(circle_with_opacity(48.0, -10.0, 6.0, "#FFFFFF", 0.7))
        .child(path_with_fill("M0 12 L-18 32 Q0 44 18 32 Z", "#F472B6"))
        .child(path_with_fill("M0 32 L-16 52 Q0 60 16 52 Z", "#FBEAED"))
        .child(path_with_fill("M0 46 Q-32 78 0 86 Q32 78 0 46", "#FCA5A5"))
        .child(
            Element::new("ellipse")
                .attr("cx", "0")
                .attr("cy", "74")
                .attr("rx", "70")
                .attr("ry", "82")
                .attr("fill", "url(#bellyGrad)")
                .empty(),
        )
        .child(path_with_fill("M-128 48 Q-176 56 -176 120 Q-128 112 -104 80", "#F7A8B8"))
        .child(path_with_fill("M128 48 Q176 56 176 120 Q128 112 104 80", "#55CDFC"))
        .child(circle_with_opacity(-100.0, 94.0, 18.0, "#FDE68A", 0.85))
        .child(circle_with_opacity(100.0, 94.0, 18.0, "#FDE68A", 0.85));

    Document::new(480, 520)
        .view_box("0 0 480 520")
        .role("img")
        .aria_label("title", "desc")
        .title("Nixette Mascot Badge")
        .desc("Chibi penguin mascot with trans flag hair, blending Nix and Gentoo motifs.")
        .add_defs(defs)
        .add_element(
            Element::new("rect")
                .attr("width", "480")
                .attr("height", "520")
                .attr("rx", "48")
                .attr("fill", "url(#bgGrad)")
                .empty(),
        )
        .add_element(body)
        .add_element(
            Group::new()
                .attr("transform", "translate(90 420)")
                .attr(
                    "font-family",
                    "'Fira Sans', 'Inter', 'Segoe UI', sans-serif",
                )
                .attr("font-size", "42")
                .attr("fill", "#E0E7FF")
                .attr("letter-spacing", "6")
                .child(Element::new("text").text("NIXIE")),
        )
        .add_element(
            Group::new()
                .attr("transform", "translate(90 468)")
                .attr(
                    "font-family",
                    "'Fira Sans', 'Inter', 'Segoe UI', sans-serif",
                )
                .attr("font-size", "20")
                .attr("fill", "#A5B4FC")
                .child(Element::new("text").text("Declarative · Sourceful · Herself")),
        )
        .finish()
}

fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64, fill: &str) -> String {
    Element::new("ellipse")
        .attr("cx", &format!("{}", cx))
        .attr("cy", &format!("{}", cy))
        .attr("rx", &format!("{}", rx))
        .attr("ry", &format!("{}", ry))
        .attr("fill", fill)
        .empty()
}

fn circle(cx: f64, cy: f64, r: f64, fill: &str) -> String {
    Element::new("circle")
        .attr("cx", &format!("{}", cx))
        .attr("cy", &format!("{}", cy))
        .attr("r", &format!("{}", r))
        .attr("fill", fill)
        .empty()
}

fn circle_with_opacity(cx: f64, cy: f64, r: f64, fill: &str, opacity: f64) -> String {
    Element::new("circle")
        .attr("cx", &format!("{}", cx))
        .attr("cy", &format!("{}", cy))
        .attr("r", &format!("{}", r))
        .attr("fill", fill)
        .attr("opacity", &format!("{}", opacity))
        .empty()
}

fn path_with_fill(d: &str, fill: &str) -> String {
    Element::new("path").attr("d", d).attr("fill", fill).empty()
}
