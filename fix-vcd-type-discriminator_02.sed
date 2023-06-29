{
 /@Override/,+2 {
    0,/public .+(Type|Value) type\(String type\) \{/ {
        /public .+(Type|Value) type\(String type\) \{/ {
            N;N;
            s/type/_&/;
            s/setType/set_Type/;
        }
    }
 }
}