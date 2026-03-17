#include <assert.h>
#include <stdio.h>
typedef struct {
  int i;
} xt;

xt x;
int global;


void f(xt i)
{
  global=1;
}

void g(xt i)
{
  global=0;
}

int main()
{
  void (*p)(xt);
  _Bool c;
  
  p=c?f:g;

  x.i=2;

  p(x);
  printf("value of c is %d", c);
  assert(global== (c?1:0));
}
