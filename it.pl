#! /usr/bin/env perl

use strict;
use feature 'say';
use autodie;


`make getbasc`;

my $examples = "examples";

my @outbin = ();

open my $out, "-|", 'ls examples';

while (my $file = <$out>) {
    $file =~ s/^\s+|\s+$//g;
    my $base = $file =~ s/^(.*).bath$/$1/r;

    print "test compile $file => $base ... ";
    system("./basc $examples/$file $base") == 0 or die;
    print "ok\n";
    push @outbin, $base;
}

# opendir my $files, $examples;

# foreach my $file (readdir $files) {
#     next if $file =~ /^\./;

#     my $base = $file =~ s/^(.*).bath$/$1/r;
#     say "test run $base";
# }

for my $bin (@outbin) {
    say "runing $bin";
    `./$bin` or die $!;
}
