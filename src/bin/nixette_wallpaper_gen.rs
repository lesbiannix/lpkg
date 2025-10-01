use anyhow::Result;
use package_management::svg_builder::{
    Defs, Document, Element, Gradient, Group, RadialGradient, path,
};
use std::fs;

fn main() -> Result<()> {
    let svg = build_wallpaper_svg();
    fs::create_dir_all("assets")?;
    fs::write("assets/nixette-wallpaper.svg", svg)?;
    Ok(())
}

fn build_wallpaper_svg() -> String {
    let defs = Defs::new()
        .linear_gradient(
            "sky",
            Gradient::new("0", "0", "1", "1")
                .stop("0%", &[("stop-color", "#0f172a")])
                .stop("100%", &[("stop-color", "#1e1b4b")]),
        )
        .linear_gradient(
            "wave1",
            Gradient::new("0", "0", "1", "0")
                .stop("0%", &[("stop-color", "#55CDFC"), ("stop-opacity", "0")])
                .stop("50%", &[("stop-color", "#55CDFC"), ("stop-opacity", "0.5")])
                .stop("100%", &[("stop-color", "#55CDFC"), ("stop-opacity", "0")]),
        )
        .linear_gradient(
            "wave2",
            Gradient::new("1", "0", "0", "0")
                .stop("0%", &[("stop-color", "#F7A8B8"), ("stop-opacity", "0")])
                .stop(
                    "50%",
                    &[("stop-color", "#F7A8B8"), ("stop-opacity", "0.55")],
                )
                .stop("100%", &[("stop-color", "#F7A8B8"), ("stop-opacity", "0")]),
        )
        .radial_gradient(
            "halo",
            RadialGradient::new("0.5", "0.5", "0.7")
                .stop("0%", &[("stop-color", "#FDE68A"), ("stop-opacity", "0.8")])
                .stop("100%", &[("stop-color", "#FDE68A"), ("stop-opacity", "0")]),
        );

    let text = Group::new()
        .attr("transform", "translate(940 1320)")
        .attr(
            "font-family",
            "'Fira Sans', 'Inter', 'Segoe UI', sans-serif",
        )
        .attr("font-size", "220")
        .attr("font-weight", "700")
        .attr("letter-spacing", "18")
        .attr("fill", "#FFFFFF")
        .attr("opacity", "0.95")
        .child(Element::new("text").text("NIXETTE"));

    let subtitle = Group::new()
        .attr("transform", "translate(960 1500)")
        .attr(
            "font-family",
            "'Fira Sans', 'Inter', 'Segoe UI', sans-serif",
        )
        .attr("font-size", "64")
        .attr("fill", "#F7A8B8")
        .attr("opacity", "0.9")
        .child(Element::new("text").text("Declarative · Sourceful · Herself"));

    Document::new(3840, 2160)
        .view_box("0 0 3840 2160")
        .role("img")
        .aria_label("title", "desc")
        .title("Nixette Wallpaper")
        .desc("Gradient wallpaper combining trans flag waves with Nix and Gentoo motifs.")
        .add_defs(defs)
        .add_element(
            Element::new("rect")
                .attr("width", "3840")
                .attr("height", "2160")
                .attr("fill", "url(#sky)")
                .empty(),
        )
        .add_element(
            Element::new("rect")
                .attr("x", "0")
                .attr("y", "0")
                .attr("width", "3840")
                .attr("height", "2160")
                .attr("fill", "url(#halo)")
                .attr("opacity", "0.4")
                .empty(),
        )
        .add_element(
            Element::new("path")
                .attr("d", "M0 1430 C640 1320 1280 1580 1860 1500 C2440 1420 3040 1660 3840 1500 L3840 2160 L0 2160 Z")
                .attr("fill", "url(#wave1)")
                .empty(),
        )
        .add_element(
            Element::new("path")
                .attr("d", "M0 1700 C500 1580 1200 1880 1900 1760 C2600 1640 3200 1920 3840 1800 L3840 2160 L0 2160 Z")
                .attr("fill", "url(#wave2)")
                .empty(),
        )
        .add_element(
            Group::new()
                .attr("opacity", "0.08")
                .attr("fill", "none")
                .attr("stroke", "#FFFFFF")
                .attr("stroke-width", "24")
                .child(path("M600 360 l220 -220 h360 l220 220 l-220 220 h-360 z"))
                .child(path("M600 360 l220 -220"))
                .child(path("M820 140 l220 220")),
        )
        .add_element(
            Group::new()
                .attr("opacity", "0.12")
                .attr("fill", "none")
                .attr("stroke", "#FFFFFF")
                .attr("stroke-width", "22")
                .attr("transform", "translate(2820 320) scale(0.9)")
                .child(path("M0 0 C120 -40 220 40 220 160 C220 260 160 320 60 320")),
        )
        .add_element(text)
        .add_element(subtitle)
        .finish()
}
