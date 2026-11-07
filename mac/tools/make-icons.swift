// Draws the menu-bar icons as template PNGs (black on clear; macOS tints them): bada ime's 바, and
// one badge per mode: A (muhenkan) and 바 (henkan) in a rounded square.
// Usage: swift make-icons.swift <out dir>
import Cocoa

let out = URL(fileURLWithPath: CommandLine.arguments[1])
func draw(_ text: String, boxed: Bool, size: CGFloat, to name: String) {
    let image = NSImage(size: NSSize(width: size, height: size), flipped: false) { rect in
        NSColor.black.set()
        if boxed {
            let box = NSBezierPath(roundedRect: rect.insetBy(dx: size * 0.06, dy: size * 0.06), xRadius: size * 0.18, yRadius: size * 0.18)
            box.lineWidth = max(1, size / 16)
            box.stroke()
        }
        let font = NSFont.systemFont(ofSize: size * (boxed ? 0.62 : 0.8), weight: .semibold)
        let attrs: [NSAttributedString.Key: Any] = [.font: font, .foregroundColor: NSColor.black]
        let s = text as NSString
        let ts = s.size(withAttributes: attrs)
        s.draw(at: NSPoint(x: (rect.width - ts.width) / 2, y: (rect.height - ts.height) / 2), withAttributes: attrs)
        return true
    }
    let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: Int(size), pixelsHigh: Int(size), bitsPerSample: 8,
                               samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
    image.draw(in: NSRect(x: 0, y: 0, width: size, height: size))
    NSGraphicsContext.restoreGraphicsState()
    try! rep.representation(using: .png, properties: [:])!.write(to: out.appendingPathComponent(name))
}
for (name, text, boxed) in [("bada", "바", false), ("muhenkan", "A", true), ("henkan", "바", true)] {
    draw(text, boxed: boxed, size: 16, to: "\(name).png")
    draw(text, boxed: boxed, size: 32, to: "\(name)@2x.png")
}
