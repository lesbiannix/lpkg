use anyhow::Result;
use package_management::svg_builder::{Defs, Document, Element, Filter, Gradient, Group, path};
use std::fs;

fn main() -> Result<()> {
    let svg = build_logo_svg();
    fs::create_dir_all("assets")?;
    fs::write("assets/logo.svg", svg)?;
    Ok(())
}

fn build_logo_svg() -> String {
    let defs = Defs::new()
        .linear_gradient(
            "bgGradient",
            Gradient::new("0", "0", "1", "1")
                .stop("0%", &[("stop-color", "#0f172a")])
                .stop("100%", &[("stop-color", "#1e293b")]),
        )
        .linear_gradient(
            "cubeGradient",
            Gradient::new("0", "0", "1", "1")
                .stop("0%", &[("stop-color", "#38bdf8")])
                .stop("100%", &[("stop-color", "#0ea5e9")]),
        )
        .linear_gradient(
            "cubeShadow",
            Gradient::new("0", "1", "1", "0")
                .stop("0%", &[("stop-color", "#0ea5e9"), ("stop-opacity", "0.4")])
                .stop("100%", &[("stop-color", "#38bdf8"), ("stop-opacity", "0.1")]),
        )
        .linear_gradient(
            "textGradient",
            Gradient::new("0", "0", "0", "1")
                .stop("0%", &[("stop-color", "#f8fafc")])
                .stop("100%", &[("stop-color", "#cbd5f5")]),
        )
        .filter(
            "glow",
            Filter::new()
                .attr("x", "-20%")
                .attr("y", "-20%")
                .attr("width", "140%")
                .attr("height", "140%")
                .raw("<feGaussianBlur stdDeviation=\"8\" result=\"blur\" />")
                .raw("<feMerge><feMergeNode in=\"blur\" /><feMergeNode in=\"SourceGraphic\" /></feMerge>"),
        );

    let cube_inner = Group::new()
        .attr("filter", "url(#glow)")
        .child(
            Element::new("path")
                .attr("d", "M222 86l86-42 86 42v96l-86 42-86-42z")
                .attr("fill", "url(#cubeGradient)")
                .empty(),
        )
        .child(
            Element::new("path")
                .attr("d", "M308 44v182l86-42V86z")
                .attr("fill", "url(#cubeShadow)")
                .empty(),
        )
        .child(
            Element::new("path")
                .attr("d", "M262 96l46-22 46 22v48l-46 22-46-22z")
                .attr("fill", "#0f172a")
                .attr("opacity", "0.85")
                .empty(),
        )
        .child(
            Element::new("path")
                .attr("d", "M308 74l32 15v32l-32 15-32-15v-32z")
                .attr("fill", "none")
                .attr("stroke", "#38bdf8")
                .attr("stroke-width", "4")
                .attr("stroke-linejoin", "round")
                .empty(),
        )
        .child(
            Element::new("path")
                .attr("d", "M308 122l-32-15")
                .attr("stroke", "#38bdf8")
                .attr("stroke-width", "4")
                .attr("stroke-linecap", "round")
                .attr("opacity", "0.6")
                .empty(),
        )
        .child(
            Element::new("path")
                .attr("d", "M308 122l32-15")
                .attr("stroke", "#38bdf8")
                .attr("stroke-width", "4")
                .attr("stroke-linecap", "round")
                .attr("opacity", "0.6")
                .empty(),
        )
        .child(
            Element::new("circle")
                .attr("cx", "276")
                .attr("cy", "107")
                .attr("r", "5")
                .attr("fill", "#38bdf8")
                .empty(),
        )
        .child(
            Element::new("circle")
                .attr("cx", "340")
                .attr("cy", "107")
                .attr("r", "5")
                .attr("fill", "#38bdf8")
                .empty(),
        );

    let cube = Group::new()
        .attr("transform", "translate(100 60)")
        .child(cube_inner);

    let circuits = Group::new()
        .attr("fill", "none")
        .attr("stroke", "#38bdf8")
        .attr("stroke-width", "3")
        .attr("stroke-linecap", "round")
        .attr("opacity", "0.55")
        .child(path("M120 78h72"))
        .child(path("M120 110h48"))
        .child(path("M120 142h64"))
        .child(path("M448 110h72"))
        .child(path("M472 142h88"))
        .child(path("M448 174h96"));

    let title_text = Group::new()
        .attr(
            "font-family",
            "'Fira Sans', 'Inter', 'Segoe UI', sans-serif",
        )
        .attr("font-weight", "600")
        .attr("font-size", "90")
        .attr("letter-spacing", "6")
        .child(
            Element::new("text")
                .attr("x", "120")
                .attr("y", "246")
                .attr("fill", "url(#textGradient)")
                .text("LPKG"),
        );

    let tagline_group = Group::new()
        .attr(
            "font-family",
            "'Fira Sans', 'Inter', 'Segoe UI', sans-serif",
        )
        .attr("font-size", "22")
        .attr("fill", "#94a3b8")
        .child(
            Element::new("text")
                .attr("x", "122")
                .attr("y", "278")
                .text("Lightweight Package Manager"),
        );

    Document::new(640, 320)
        .view_box("0 0 640 320")
        .role("img")
        .aria_label("title", "desc")
        .title("LPKG Logo")
        .desc("Stylised package icon with circuitry and the letters LPKG.")
        .add_defs(defs)
        .add_element(
            Element::new("rect")
                .attr("width", "640")
                .attr("height", "320")
                .attr("rx", "28")
                .attr("fill", "url(#bgGradient)")
                .empty(),
        )
        .add_element(cube)
        .add_element(circuits)
        .add_element(title_text)
        .add_element(tagline_group)
        .finish()
}
