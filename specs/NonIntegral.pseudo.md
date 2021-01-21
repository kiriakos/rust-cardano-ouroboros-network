# Pseudocode for ln implementation of NonIntegral.hs

source: `cardano-ledger-specs/shelley/chain-and-ledger/dependencies/non-integer/NonIntegral.hs`

pseudocode impl of `ln' x` all assignments are value based except assignment of
functions.

## NOTE: WIP!

The logarithm e base impl:
```
ln' (x) {

  if(x <= 0){ 
    print("X must be positive, non zero");
    exit(1);
  }

  var (integral, fractional) = splitLn(x);

  return integral + lncf(1000,fractional);
}
```

Split fractional exponent into two parts
an `integer` part and a `fractional` part
```
splitLn (x) {

  if(x < 0){ 
    print("X must be positive");
    exit(1);
  }

  var integral;
  var fractional;
 
  integral = findE(exp1, x);
  fractional = (x/(ipow(exp1, integral))) - 1;
  
  return (integral, fractional);
}

const exp1 =  exp'(1);
```

Exponentiation functions
```
exp' (x){

  if ( x<0 ){
    return 1 / exp'(-x);
  }
  else{

    var (n, x_) = scaleExp(x);
    var x' = taylorExp(1000, 1, x_, 1, 1, 1);

    return ipow(x', n);
  }
}

taylorExp (maxN, n, x, lastX, acc, divisor){

  if(maxN == n || abs(nextX) < eps){
    return acc;
  }else{

    var nextX = (lastX * x)/divisor # NOTE: nobody checks for `divisor == 0`...
    
    n+=1;
    divisor+=1;
    acc = acc + nextX;

    return taylorExp(maxN, n, x, nextX, acc, divisor); 
    # TODO: recursion to loop
  }
  
}

findE(e,x){

  var (lower, upper) = bound e x (1/e) e (-1) 1
  return contract() 

}
```

Contraction

OK, this one is a bit weird. Its implementation returns a
two argument function `go` that is applied to the last two
arguments of `contract` to produce the integer result.
```
contract (factor, x, l, u){
  if(l + 1 == u){
    return
  }

}
```
