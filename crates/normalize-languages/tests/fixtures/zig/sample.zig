const std = @import("std");
const math = @import("math_utils.zig");

pub const Point = struct {
    x: f64,
    y: f64,

    pub fn distance(self: Point, other: Point) f64 {
        const dx = self.x - other.x;
        const dy = self.y - other.dy;
        return math.sqrt(dx * dx + dy * dy);
    }

    pub fn origin() Point {
        return Point{ .x = 0.0, .y = 0.0 };
    }
};

/// Classify a number as negative, zero, or positive.
pub fn classify(n: i32) []const u8 {
    if (n < 0) {
        return "negative";
    } else if (n == 0) {
        return "zero";
    } else {
        return "positive";
    }
}

pub fn sumSlice(items: []const i32) i32 {
    var total: i32 = 0;
    for (items) |item| {
        total += item;
    }
    return total;
}

pub fn fibonacci(n: u32) u32 {
    if (n <= 1) return n;
    var a: u32 = 0;
    var b: u32 = 1;
    var i: u32 = 2;
    while (i <= n) : (i += 1) {
        const tmp = a + b;
        a = b;
        b = tmp;
    }
    return b;
}

/// Describe a digit using a switch expression, exercising SwitchExpr.
pub fn describeDigit(n: u8) []const u8 {
    switch (n) {
        0 => return "zero",
        1, 2 => return "small",
        else => return "other",
    }
}

/// Parse a number, exercising the catch error-handling suffix.
pub fn parseOrDefault(s: []const u8) i32 {
    const value = std.fmt.parseInt(i32, s, 10) catch |err| {
        std.debug.print("parse error: {}\n", .{err});
        return -1;
    };
    return value;
}

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();
    const p = Point.origin();
    try stdout.print("origin: ({d}, {d})\n", .{ p.x, p.y });
    const result = classify(-5);
    try stdout.print("classify(-5) = {s}\n", .{result});
    const total = sumSlice(&[_]i32{ 1, 2, 3, 4, 5 });
    try stdout.print("sum = {d}\n", .{total});
}
