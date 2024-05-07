#include <stdio.h>

int fuck(int a) {
	int b = 2 + a;
	return b;
}

int main() {
	int f = fuck(10) * 2;
	printf("%d\n", f);
	return 0;
}