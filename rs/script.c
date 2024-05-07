#include <stdio.h>

int fuck(int a) {
    int b = 2 + a;
    if (b < 10) {
        return 0;
    } else {
        return b;
    }
}

int main() {
    int f = fuck(10) * 2;
    printf("%d\n", f);
    return 0;
}