{
    /JSON_PROPERTY_TYPE = "_type"/,+1 {
        s/type/_&/i;
    };
    0,/(\.JSON_PROPERTY_)(TYPE,?)$/s//\1_\2/;
    0,/Objects\.equals\(this\.type,/ {
        /Objects\.equals\(this\.type,/s/type/_&/g;
    };
    0,/\.append\(toIndentedString\(type\)\)/ {
        /\.append\(toIndentedString\(type\)\)/s/type/_&/g;
    };
    /Objects\.hash/s/type[,\)]/_&/;
    0,/type\([a-zA-Z]+ type\) \{/ {
        /type\([a-zA-Z]+ type\) \{/ {
            N;N;
            s/type/_&/;
            s/\.(type)/\._\1/;
        }
    };
    0,/\(JSON_PROPERTY_TYPE\)/ {
        /\(JSON_PROPERTY_TYPE\)/ {
                N;N;N;N;N;
                s/type/_&/i;
                s/type = type/_&/;
                s/(return )(type)/\1_\2/;
                s/getType/get_Type/;
                s/setType/set_Type/;
            }
     };
     0,/\(JSON_PROPERTY_TYPE\)/ {
        /\(JSON_PROPERTY_TYPE\)/ {
                N;N;N;N;N;
                s/type/_&/i;
                s/type = type/_&/;
                s/(return )(type)/\1_\2/;
                s/getType/get_Type/;
                s/setType/set_Type/;
            }
     };
}