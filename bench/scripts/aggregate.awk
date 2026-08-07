BEGIN { FS = "\"score\":" }
{ split($2, a, ","); s += a[1]+0 }
END { printf "%.2f\n", s }
