#include <assert.h>
int main() {
  int a;
  if (a) {
    a = 1;
  }
  else
    a = 2;

  assert(a == 1 || a == 3);
  return 0;
}

