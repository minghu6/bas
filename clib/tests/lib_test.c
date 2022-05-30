#include <stdio.h>
#include <string.h>
#include "../include/vec.h"
#include "../include/aa.h"
#include "../include/cstr.h"
#include "../include/lib.h"

#define ps(s) printf("%s\n", s);

#define test_aa(aa, s)\
  if ( (res = aa_search(aa, s, &idx)) >= 0 ) {\
    printf("%s (%ld): %d\n", s, idx, aa_get_i32(aa, idx));\
  }\
  else\
    printf("%s: no result\n", s);\

void vec_i32_print_range(Vec v, size_t start, size_t end) {
  for (int i=start; i<end;i++) {
      printf("%d: %d\n", i, vec_get_i32(v, i));
  }
}

void vec_str_print_range(Vec v, size_t start, size_t end) {
  for (int i=start; i<end;i++) {
      printf("%d: %s\n", i, (char*) vec_get_ptr(v, i));
  }
}

void test_pat() {
  const char* pat = "\\$([[[:alpha:]]_][[:alnum:]]*)";
}

void test_sym_replace() {
  char* src = "echo -n $count >> $b4";

  Vec syms = vec_new_ptr(2);
  vec_push_ptr(syms, "count");
  vec_push_ptr(syms, "b4");

  Vec repls = vec_new_ptr(2);
  vec_push_ptr(repls, "2");
  vec_push_ptr(repls, "file.txt");

  char* res = cmd_symbols_replace(src, syms, repls);
  ps(res);
}


void test_sh_call() {
  char* res;
  res = exec("sh -c ls -a");
  ps(res)
}

void test_cstr() {
  CStr str = cstr_new("abc");
  cstr_push(str, "bbb");
  cstr_push(str, "cde");

  // ps(cstr_into(str));

  ps(cstr_as_str(str));
  ps(cstr_as_str(str));

}


#define _is_ident_head(c) \
  (('a' <= c && c <= 'z' || 'A' <= c && c <= 'Z' || c == '_') ? true : false)
#define _is_ident_nonhead(c) \
  ((_is_ident_head(c) || '0' <= c && c <= '9') ? true : false)

int main()
{

  // char* s0 = "abcd";

  // ps(strndup(s0, 3));

  // if (!_is_ident_nonhead(' ')) {
  //   ps("!!!!!!!!!!!!!!!!!!!");
  // }

  // /*
  // * Test Vec
  // **/
  // Vec v = vec_new_i32(0);

  // vec_push_i32(v, 12);
  // vec_push_i32(v, 24);

  // vec_i32_print_range(v, 0, 3);
  // ps("\n");
  // vec_insert_i32(v, 0, 111);
  // vec_insert_i32(v, 3, 33);

  // vec_i32_print_range(v, 0, 5);

  // Vec v2 = vec_new_ptr(3);
  // vec_push_ptr(v2, "aaa");
  // vec_push_ptr(v2, "你好");
  // vec_str_print_range(v2, 0, 2);



  // /*
  // * Test AA
  // **/
  // AssocArr aa = aa_new_i32(5);

  // aa_insert_i32(aa, "wowo", 40);
  // aa_insert_i32(aa, "coco", 60);
  // aa_insert_i32(aa, "aaaa", 20);

  // size_t idx;
  // int res;

  // test_aa(aa, "aaaa");
  // test_aa(aa, "wowo");
  // test_aa(aa, "coco");
  // test_aa(aa, "++++");

  // printf("%lf\n", 12.34);
  // printf("%d\n", 12);
  // printf("%d\n", true);
  // printf("%d\n", false);

  // printf("%s\n", stringify_f64(12.34));
  // printf("%s\n", stringify_i32(12));

  // char* s = strdup("12.3444");
  // printf("%s\n", s);
  // free(s);

  test_sym_replace();

  test_sh_call();

  test_cstr();

  return 0;
}
