const std = @import("std");

const DNSHeader = struct {
    id: u16,
    flags: u16,
    num_questions: u16 = 0,
    num_answers: u16 = 0,
    num_authority: u16 = 0,
    num_additionals: u16 = 0,

    fn to_bytes(self: DNSHeader) [@sizeOf(DNSHeader)]u8 {
        var buf: [@sizeOf(DNSHeader)]u8 = undefined;
        buf[0..2].* = @bitCast([2]u8, self.id);
        buf[2..4].* = @bitCast([2]u8, self.flags);
        buf[4..6].* = @bitCast([2]u8, self.num_questions);
        buf[6..8].* = @bitCast([2]u8, self.num_answers);
        buf[8..10].* = @bitCast([2]u8, self.num_authority);
        buf[10..12].* = @bitCast([2]u8, self.num_additionals);
        return buf;
    }
};
const ArrayList = std.ArrayList;

const DNSQuestion = struct {
    name: []u8,
    type: u16,
    class: u16,

    const Self = @This();

    fn to_bytes(self: Self, allocator: std.mem.Allocator) error{OutOfMemory}![]u8 {
        var array = ArrayList(u8).init(allocator);
        errdefer array.deinit();

        try array.appendSlice(self.name);
        try array.appendSlice(&std.mem.toBytes(self.type));
        try array.appendSlice(&std.mem.toBytes(self.class));

        return array.toOwnedSlice();
    }
};

pub fn main() !void {
    var buffer: [200]u8 = undefined;
    var fba = std.heap.FixedBufferAllocator.init(&buffer);
    const allocator = fba.allocator();

    const header = DNSHeader{
        .id = 1,
        .flags = 2,
    };
    std.debug.print("{any} {any} {any} {any} {any} {any}", header);
    std.debug.print("{any}", .{header.to_bytes()});

    var domain_array = ArrayList(u8).init(allocator);
    errdefer domain_array.deinit();
    try domain_array.appendSlice("www.google.com");

    const question = DNSQuestion{
        .name = domain_array.items,
        .type = 1,
        .class = 5,
    };

    std.debug.print("{any}", .{try question.to_bytes(allocator)});
}
