#include <stdio.h>

int isPalindrome(int n) {
    int rev = 0;
    int temp = n;
    while (temp > 0) {
        rev = rev * 10 + temp % 10;
        temp = temp / 10;
    };
    return n == rev;
}

int main() {
    int max = 0;
    int a = 1;
    while (a < 1000) {
        int b = a;
        while (b < 1000) {
            if (isPalindrome(a * b) && a * b > max) {
                max = a * b;
            };
            b += 1;
        };
        a += 1;
    };
    printf("%d\n", max);
    return 0;
}