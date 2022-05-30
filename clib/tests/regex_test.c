/* regex_replace.c
:w | !gcc % -o .%<
:w | !gcc % -o .%< && ./.%<
:w | !gcc % -o .%< && valgrind -v ./.%<
*/
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <regex.h>

int regex_replace(char **str, const char *pattern, const char *replace)
{
  // replaces regex in pattern with replacement observing capture groups
  // *str MUST be free-able, i.e. obtained by strdup, malloc, ...
  // back references are indicated by char codes 1-31 and none of those chars can be used in the replacement string such as a tab.
  // will not search for matches within replaced text, this will begin searching for the next match after the end of prev match
  // returns:
  //   -1 if pattern cannot be compiled
  //   -2 if count of back references and capture groups don't match
  //   otherwise returns number of matches that were found and replaced
  //
  regex_t reg;
  unsigned int replacements = 0;
  // if regex can't commpile pattern, do nothing
  if (!regcomp(&reg, pattern, REG_EXTENDED))
  {
    size_t nmatch = reg.re_nsub;
    regmatch_t m[nmatch + 1];
    const char *rpl, *p;
    // count back references in replace
    int br = 0;
    p = replace;
    while (1)
    {
      while (*++p > 31)
        ;
      if (*p)
        br++;
      else
        break;
    } // if br is not equal to nmatch, leave
    if (br != nmatch)
    {
      regfree(&reg);
      return -2;
    }
    // look for matches and replace
    char *new;
    char *search_start = *str;
    int eflags = 0;
    // eflags |= REG_NOTBOL;

    while (!regexec(&reg, search_start, nmatch + 1, m, eflags))
    {
      // make enough room
      new = (char *)malloc(strlen(*str) + strlen(replace));
      if (!new)
        exit(EXIT_FAILURE);
      *new = '\0';
      strncat(new, *str, search_start - *str);
      p = rpl = replace;
      int c;
      strncat(new, search_start, m[0].rm_so); // test before pattern
      for (int k = 0; k < nmatch; k++)
      {
        while (*++p > 31)
          ;                         // skip printable char
        c = *p;                     // back reference (e.g. \1, \2, ...)
        strncat(new, rpl, p - rpl); // add head of rpl
        // concat match
        strncat(new, search_start + m[c].rm_so, m[c].rm_eo - m[c].rm_so);
        rpl = p++; // skip back reference, next match
      }
      strcat(new, p); // trailing of rpl
      unsigned int new_start_offset = strlen(new);
      strcat(new, search_start + m[0].rm_eo); // trailing text in *str
      free(*str);
      *str = (char *)malloc(strlen(new) + 1);
      strcpy(*str, new);
      search_start = *str + new_start_offset;
      free(new);
      replacements++;
    }
    regfree(&reg);
    // ajust size
    *str = (char *)realloc(*str, strlen(*str) + 1);
    return replacements;
  }
  else
  {
    return -1;
  }
}

// int regex_replace_n(char **str, const char *pat, __uint32_t repn, const char **replace_n)
// {
//   // replaces regex in pattern with replacement observing capture groups
//   // *str MUST be free-able, i.e. obtained by strdup, malloc, ...
//   // back references are indicated by char codes 1-31 and none of those chars can be used in the replacement string such as a tab.
//   // will not search for matches within replaced text, this will begin searching for the next match after the end of prev match
//   // returns:
//   //   -1 if pattern cannot be compiled
//   //   -2 if count of back references and capture groups don't match
//   //   otherwise returns number of matches that were found and replaced
//   //
//   regex_t reg;
//   unsigned int replacements = 0;
//   // if regex can't commpile pattern, do nothing
//   if (regcomp(&reg, pat, REG_EXTENDED))
//     return -1;

//   size_t nmatch = reg.re_nsub;
//   regmatch_t m[nmatch + 1];
//   const char *rpl, *p;

//   // count back references in replace
//   int br = 0;
//   p = repn;
//   while (1)
//   {
//     while (*++p > 31)
//       ;
//     if (*p)
//       br++;
//     else
//       break;
//   } // if br is not equal to nmatch, leave
//   if (br != nmatch)
//   {
//     regfree(&reg);
//     return -2;
//   }
//   // look for matches and replace
//   char *new;
//   char *search_start = *str;
//   int eflags = 0;
//   // eflags |= REG_NOTBOL;

//   while (!regexec(&reg, search_start, nmatch + 1, m, eflags))
//   {
//     // make enough room
//     new = (char *)malloc(strlen(*str) + strlen(repn));
//     if (!new)
//       exit(EXIT_FAILURE);
//     *new = '\0';
//     strncat(new, *str, search_start - *str);
//     p = rpl = repn;
//     int c;
//     strncat(new, search_start, m[0].rm_so); // test before pattern
//     for (int k = 0; k < nmatch; k++)
//     {
//       while (*++p > 31)
//         ;                         // skip printable char
//       c = *p;                     // back reference (e.g. \1, \2, ...)
//       strncat(new, rpl, p - rpl); // add head of rpl
//       // concat match
//       strncat(new, search_start + m[c].rm_so, m[c].rm_eo - m[c].rm_so);
//       rpl = p++; // skip back reference, next match
//     }
//     strcat(new, p); // trailing of rpl
//     unsigned int new_start_offset = strlen(new);
//     strcat(new, search_start + m[0].rm_eo); // trailing text in *str
//     free(*str);
//     *str = (char *)malloc(strlen(new) + 1);
//     strcpy(*str, new);
//     search_start = *str + new_start_offset;
//     free(new);
//     replacements++;
//   }
//   regfree(&reg);
//   // ajust size
//   *str = (char *)realloc(*str, strlen(*str) + 1);
//   return replacements;
// }

#define array_len(arr) (sizeof arr / sizeof *arr)

const char test1[] = "before [link->address] some text [link2->addr2] trail[a->[b->c]]";
const char *pattern1 = "\\[([^-]+)->([^]]+)\\]";
const char replace1[] = "<a href=\"\2\">\1</a>";

const char test2[] = "abcabcdefghijklmnopqurstuvwxyzabc";
const char *pattern2 = "abc";
const char replace2[] = "!abc";

const char test3[] = "a1a1a1a2ba1";
const char *pattern3 = "a";
const char replace3[] = "aa";

const char test4[] = "echo -n $count >> $b4";
// const char *pattern4 = "\\$[a-z_][a-zA-Z0-9]*";
const char *pattern4 = "\\$[[:alpha:]_][[:alnum:]]*";
const char replace4[] = "333";

int main(int argc, char *argv[])
{
  unsigned int repl_count;
  // const char s1[] = "echo";
  // printf("array_len: %ld, strlen: %ld\n", sizeof s1, strlen(s1));

  // char *str1 = (char *)malloc(strlen(test1)+1);
  // strcpy(str1,test1);
  // puts(str1);
  // printf("test 1 Before: [%s], ",str1);
  // unsigned int repl_count1 = regex_replace(&str1, pattern1, replace1);
  // printf("After replacing %d matches: [%s]\n",repl_count1,str1);
  // free(str1);

  // char *str2 = (char *)malloc(strlen(test2)+1);
  // strcpy(str2,test2);
  // puts(str2);
  // printf("test 2 Before: [%s], ",str2);
  // unsigned int repl_count2 = regex_replace(&str2, pattern2, replace2);
  // printf("After replacing %d matches: [%s]\n",repl_count2,str2);
  // free(str2);

  // char *str3 = (char *)malloc(strlen(test3) + 1);
  // strcpy(str3, test3);
  // puts(str3);
  // printf("test 3 Before: [%s], ", str3);
  // unsigned int repl_count3 = regex_replace(&str3, pattern3, replace3);
  // printf("After replacing %d matches: [%s]\n", repl_count3, str3);
  // free(str3);

  // char* str4 = (char *)malloc(array_len(test4));
  // strcpy(str4, test4);
  // char *str4 = strdup(test4);

  // printf("test4 origin: [%s]\n", str4);
  // repl_count = regex_replace_all(&str4, pattern4, replace4);
  // printf("replacing %d matches: [%s]\n", repl_count, str4);

  // free(str4);

  int a = 2;

  if (1 <= a <= 3) {
    printf("hi\n");
  }
}
