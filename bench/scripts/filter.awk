BEGIN { FS = "\"score\":" }
{ split($2, a, ","); if (a[1]+0 > 500) c++ }
END { print c+0 }
