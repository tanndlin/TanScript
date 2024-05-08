#include <stdio.h>

int fuck(int a) {
    int b = 2 + a;
    if (b < 10) {
        return 0;
    } else {
        return b;
    };
}
int foo(int n) {
    while (n < 10) {
        printf("%d\n", n);
        n = n + 1;
    };
}

int main() {
    int f = fuck(10) * 2;
    printf("%d\n", f);
    foo(0);
    return 0;
}