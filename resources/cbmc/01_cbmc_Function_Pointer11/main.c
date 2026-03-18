unsigned fritz(unsigned x) { return x+1; }
unsigned franz(unsigned x) { return x; }

typedef unsigned (fun_t)(unsigned);

fun_t *f;

int main()
{
  unsigned x = nondet_uint();
  if(!(x>20)) return 0;
  if(x<10)
    f=0;
  else
    f=franz;

  // should be fine, and pass without warning
  f(x);
} 
