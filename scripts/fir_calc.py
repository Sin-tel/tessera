import numpy as np

k_taps = 8
ntaps = k_taps * 4 - 1
beta = 8  # 6

print("n taps", ntaps)

N = ntaps - 1

# we do a divide by zero but fix it later
np.seterr(divide="ignore", invalid="ignore")

if ntaps % 2 == 0:
    print("ERROR ntaps must be odd")

w = np.kaiser(ntaps, beta)
x = np.linspace(-N / 2, N / 2, ntaps)


sinc = np.sin(np.pi * x / 2) / (np.pi * x)

# fix divide by zero
sinc[N // 2] = 1 / 2

out = np.multiply(w, sinc)

tol = 1e-16
out[abs(out) < tol] = 0.0


print("sum:", np.sum(out))

out = out.astype(np.float32)

print("downsampler coefs:")
print(
    np.array2string(
        out[0 : N // 2 : 2],
        separator=", ",
        floatmode="unique",
        max_line_width=5,
    )
)
print("upsampler coefs:")
print(
    np.array2string(2.0 * out[0 : N // 2 : 2], separator=", ", floatmode="unique", max_line_width=5)
)
