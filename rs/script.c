#include <stdio.h>



int main() {
	int a = 4;
	int b = 5;
	int c = ~a ^ b;
	printf("%d\n", c);
	return 0;
}