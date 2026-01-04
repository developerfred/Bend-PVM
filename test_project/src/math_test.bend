#{MathTest}
# Test the Math standard library functions

def test_sin() -> f24:
    return Math/sin(0.0)

def test_cos() -> f24:
    return Math/cos(0.0)

def test_sqrt() -> f24:
    return Math/sqrt(4.0)

def test_min() -> f24:
    return Math/min(1.0, 2.0)

def test_max() -> f24:
    return Math/max(1.0, 2.0)

def test_pow() -> f24:
    return Math/pow(2.0, 3.0)

def main() -> u24:
    let s = test_sin()
    let c = test_cos()
    let sq = test_sqrt()
    let mn = test_min()
    let mx = test_max()
    let pw = test_pow()
    return 42