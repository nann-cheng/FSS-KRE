import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator

# display execuation time figure or communication volume figure
is_show_time_figure=False

if is_show_time_figure:
    x = [100, 1000, 10000, 100000, 1000000]
    y1 = [4.23, 4.26, 4.34, 5.11, 10.92]#commu: [487,2174,19049,187799,1875299] (bitMax, n=15)
    y2 = [1.98, 2.01, 2.24, 3.76, 23.69]#commu: [458,2145,19020,187770,1875270] (batchMax, \omega=3)
    y3 = [8.50,8.52,8.69,10.03,21.40]#commu: [989,4364,38114,375614,3750614] (bitMax, n=30)
    y4 = [3.88,3.92,4.02,7.66,46.56]#commu: [915,4290,38040,375540,3750540] (batchMax, \omega=3)

    fig, ax = plt.subplots(figsize=(5, 2.7), layout='constrained')
    ax.plot(x, y1, '-',label='Max1')
    ax.plot(x, y2, '-.', label='Max2 $(\omega = 3)$')

    ax.set_xlabel('Input scale')
    ax.set_ylabel('Estimated execution time [s]')
    ax.xaxis.set_major_locator(MaxNLocator(integer=True))
    ax.legend()
    plt.show()
else:
    x = [100, 1000, 10000, 100000, 1000000]
    y1 = [487,2174,19049,187799,1875299] #(bitMax, n=15)
    y2 = [458,2145,19020,187770,1875270] #(batchMax, \omega=3)
    y3 = [989,4364,38114,375614,3750614] #(bitMax, n=30)
    y4 = [915,4290,38040,375540,3750540] #(batchMax, \omega=3)

    y1 = [e/1000 for e in y1]
    y2 = [e/1000 for e in y2]

    fig, ax = plt.subplots(figsize=(5, 2.7), layout='constrained')
    ax.plot(x, y1, '-',label='Max1')
    ax.plot(x, y2, '-.', label='Max2 $(\omega = 3)$')

    ax.set_xlabel('Input scale')
    ax.set_ylabel('Estimated execution time [s]')
    ax.xaxis.set_major_locator(MaxNLocator(integer=True))
    ax.legend()
    plt.show()
