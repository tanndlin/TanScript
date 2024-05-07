#include <stdio.h>

int fuck(int a) {
	int b = 2 + a;
	if (b) {
	return 0;
} else {
	return 1;
}
	return b;
}

int main() {
	int f = fuck(10) * 2;
	printf("%d\n", f);
	return 0;
}