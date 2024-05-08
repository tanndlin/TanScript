#include <stdio.h>

int main() {
    int sum = 0;
    int n = 1;
    while (n < 1000) {
        if (n % 3 == 0 || n % 5 == 0) {
            sum = sum + n;
        };
        n = n + 1;
    };
    printf("%d\n", sum);
    return 0;
}