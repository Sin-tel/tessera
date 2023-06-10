import numpy as np

# frames per wavetable
m = 16
# size of each frame
n = 1024


w = np.linspace(0, 1.0, n, endpoint=False, dtype=np.float64)

wt = np.zeros(shape=(m, n))

# for i,v in enumerate(wt):
#     if i > 0:
#         p = i
#         wt[i] = np.sin(2 * np.pi * (np.exp(p*w) - 1.0)/(np.exp(p) - 1.0))
#     else:
#         wt[i] = np.sin(2 * np.pi * w)


# for i,v in enumerate(wt):
#     p = i * 0.5 + 0.5
#     wt[i] = np.sin(p * np.sin(2*np.pi * w) + 0.5)

# for i,v in enumerate(wt):
#     p = i * 0.15 + 0.5
#     wt[i] = np.sin(p * np.sin(2*np.pi*w + p * np.sin(2*np.pi*w)) + 0.5)

for i, v in enumerate(wt):
    p = (i / 16) * 4.3 + 0.68
    wt[i] = np.sin(np.pi * w) * np.sin(2 * np.pi * p * w)


# remove DC
for i, v in enumerate(wt):
    wt[i] = wt[i] - np.average(wt[i])

# normalize
for i, v in enumerate(wt):
    wt[i] = wt[i] / np.max(np.abs(wt[i]))


f32 = np.dtype("<f")  # little-endian single-precision float

out = wt.flatten().astype(f32)

with open("../lua/res/wavetable.bin", "wb") as f:
    out.tofile(f)
