#include <stdint.h>

#pragma pack(push, 1)
typedef struct {
    int32_t x;
    int32_t y;
} Pair;

typedef struct {
    uint8_t a;
    uint8_t b;
    uint8_t c;
    uint8_t d;
} Quad;

typedef struct {
    uint8_t a;
    uint8_t b;
} Bytes;

typedef struct {
    uint8_t v;
} One;

typedef struct {
    int64_t x;
    int32_t y;
    uint8_t z;
} Point;

typedef struct {
    uint8_t a;
    uint8_t b;
    uint8_t c;
} Three;
#pragma pack(pop)

int64_t c_sum_pair(Pair p) {
    return (int64_t)p.x + p.y;
}

int64_t c_sum_quad(Quad q) {
    return q.a + q.b + q.c + q.d;
}

int64_t c_sum_bytes(Bytes b) {
    return b.a + b.b;
}

int64_t c_get_one(One o) {
    return o.v;
}

int64_t c_bump_point(Point p) {
    p.x += 100;
    p.y += 10;
    p.z += 1;
    return p.x + p.y + p.z;
}

int64_t c_bump_three(Three t) {
    t.a += 1;
    t.b += 1;
    t.c += 1;
    return t.a + t.b + t.c;
}

int64_t c_mixed(One o, Bytes b, Point p, Quad q, Pair last, Three t) {
    int64_t total = o.v + b.a + b.b + p.x + p.y + p.z + q.a + q.b + q.c + q.d + last.x + last.y
        + t.a + t.b + t.c;
    p.x = 0;
    t.a = 0;
    return total;
}
