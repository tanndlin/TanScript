#include <stdio.h>
#include <stdbool.h>

int add(int a, int b) {
	return a + b;
}

int main() {
	int a = 1;
	int b = 2;
	printf("%d\n", add(a, b));
	return 0;
}