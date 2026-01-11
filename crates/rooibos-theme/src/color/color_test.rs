use super::Color;

#[test]
fn convert() {
    let color: Color = "oklch(64.74% 0.039 281.8);".parse().unwrap();
    assert_eq!(color.to_string(), "rgb(138 140 166)");
    assert_eq!(color.to_hex_fg(), "#8A8CA6");
}
