use anyhow::Result;
use package_management::svg_builder::{Defs, Document, Element, Filter, Gradient, Group};
use std::fs;

fn main() -> Result<()> {
    let svg = build_nixette_logo();
    fs::create_dir_all("assets")?;
    fs::write("assets/nixette-logo.svg", svg)?;
    Ok(())
}

fn build_nixette_logo() -> String {
    let defs = Defs::new()
        .linear_gradient(
            "bg",
            Gradient::new("0", "0", "1", "1")
                .stop("0%", &[("stop-color", "#55CDFC")])
                .stop("100%", &[("stop-color", "#F7A8B8")]),
        )
        .linear_gradient(
            "text",
            Gradient::new("0", "0", "0", "1")
                .stop("0%", &[("stop-color", "#FFFFFF")])
                .stop("100%", &[("stop-color", "#E5E7FF")]),
        )
        .filter(
            "softShadow",
            Filter::new()
                .attr("x", "-10%")
                .attr("y", "-10%")
                .attr("width", "120%")
                .attr("height", "120%")
                .raw("<feDropShadow dx=\"0\" dy=\"6\" stdDeviation=\"12\" flood-color=\"#7C3AED\" flood-opacity=\"0.3\" />"),
        );

    let emblem = Group::new().attr("transform", "translate(100 60)").child(
        Group::new()
            .attr("filter", "url(#softShadow)")
            .child(
                Element::new("path")
                    .attr("d", "M40 40 L72 0 L144 0 L176 40 L144 80 L72 80 Z")
                    .attr("fill", "url(#bg)")
                    .empty(),
            )
            .child(
                Element::new("path")
                    .attr("d", "M72 0 L144 80")
                    .attr("stroke", "#FFFFFF")
                    .attr("stroke-width", "6")
                    .attr("stroke-linecap", "round")
                    .attr("opacity", "0.55")
                    .empty(),
            )
            .child(
                Element::new("path")
                    .attr("d", "M144 0 L72 80")
                    .attr("stroke", "#FFFFFF")
                    .attr("stroke-width", "6")
                    .attr("stroke-linecap", "round")
                    .attr("opacity", "0.55")
                    .empty(),
            )
            .child(
                Element::new("circle")
                    .attr("cx", "108")
                    .attr("cy", "40")
                    .attr("r", "22")
                    .attr("fill", "#0F172A")
                    .attr("stroke", "#FFFFFF")
                    .attr("stroke-width", "6")
                    .attr("opacity", "0.85")
                    .empty(),
            )
            .child(
                Element::new("path")
                    .attr("d", "M108 24c8 0 14 6 14 16s-6 16-14 16")
                    .attr("stroke", "#F7A8B8")
                    .attr("stroke-width", "4")
                    .attr("stroke-linecap", "round")
                    .attr("fill", "none")
                    .empty(),
            ),
    );

    let wordmark = Group::new()
        .attr("transform", "translate(220 126)")
        .attr(
            "font-family",
            "'Fira Sans', 'Inter', 'Segoe UI', sans-serif",
        )
        .attr("font-weight", "700")
        .attr("font-size", "72")
        .attr("letter-spacing", "4")
        .attr("fill", "url(#text)")
        .child(Element::new("text").text("NIXETTE"));

    let subtitle = Group::new()
        .attr("transform", "translate(220 160)")
        .attr(
            "font-family",
            "'Fira Sans', 'Inter', 'Segoe UI', sans-serif",
        )
        .attr("font-size", "22")
        .attr("fill", "#A5B4FC")
        .child(Element::new("text").text("Declarative · Sourceful · Herself"));

    Document::new(640, 200)
        .view_box("0 0 640 200")
        .role("img")
        .aria_label("title", "desc")
        .title("Nixette Logo")
        .desc("Wordmark combining Nix and Gentoo motifs with trans pride colours.")
        .add_defs(defs)
        .add_element(
            Element::new("rect")
                .attr("width", "640")
                .attr("height", "200")
                .attr("rx", "36")
                .attr("fill", "#0F172A")
                .empty(),
        )
        .add_element(emblem)
        .add_element(wordmark)
        .add_element(subtitle)
        .finish()
}
