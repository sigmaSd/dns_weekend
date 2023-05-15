const std = @import("std");
const Allocator = std.mem.Allocator;

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

    fn to_bytes(self: Self, allocator: Allocator) error{OutOfMemory}![]u8 {
        var array = ArrayList(u8).init(allocator);
        errdefer array.deinit();

        try array.appendSlice(self.name);
        try array.appendSlice(&std.mem.toBytes(self.type));
        try array.appendSlice(&std.mem.toBytes(self.class));

        return array.toOwnedSlice();
    }
};

fn create_struct(n: comptime_int) type {
    return struct { a: [n]u8 };
}

fn encode_dns_name(domain_name: []const u8, allocator: Allocator) error{OutOfMemory}![]const u8 {
    var array = ArrayList(u8).init(allocator);
    errdefer array.deinit();
    var iter = std.mem.split(u8, domain_name, &[_]u8{'.'});
    while (iter.next()) |part| {
        try array.append(std.mem.toBytes(part.len)[0]);
        try array.appendSlice(part);
    }
    try array.append(0x0);

    return array.toOwnedSlice();
}

const test_allocator = std.testing.allocator;
test "encode_dns_name_test" {
    const expectEqualSlices = std.testing.expectEqualSlices;
    const encoded = try encode_dns_name("google.com", test_allocator);
    defer test_allocator.free(encoded);
    try expectEqualSlices(u8, "\x06google\x03com\x00", encoded);
    try expectEqualSlices(u8, "\x06google\x03com\x00", &encode_dns_name_comptime(10, "google.com".*));
}

fn encode_dns_name_comptime(comptime len: usize, comptime domain_name: [len]u8) [domain_name.len + 2]u8 {
    var encoded: [domain_name.len + 2]u8 = undefined;
    var offset: usize = 0;
    var iter = std.mem.split(u8, &domain_name, &[_]u8{'.'});

    while (iter.next()) |part| {
        encoded[offset] = std.mem.toBytes(part.len)[0];
        offset += 1;

        for (part) |element, i| {
            encoded[offset + i] = element;
        }

        offset += part.len;
    }
    encoded[offset] = 0x0;

    return encoded;
}

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
