import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator

# display execuation time figure or communication volume figure
is_show_time_figure=False

if is_show_time_figure:
    x = [100, 1000, 10000, 100000, 1000000]
    y1 = [4.00, 4.03, 4.00, 4.73, 10.27]
    y2 = [1.79, 1.91, 2.00, 3.38, 16.21]

    y3 = [7.48, 7.97, 7.54, 9.23, 20.59]
    y4 = [3.45, 3.47, 3.92, 6.68, 33.83]

    plt.figure(figsize=(5, 2.7), layout='constrained')

    # Create scatter plots
    plt.scatter(x, y1, label='y1 (bitMax, n=15)', marker='o')
    plt.scatter(x, y2, label='y2 (batchMax, n=15, ω=3)', marker='x')
    plt.scatter(x, y3, label='y3 (bitMax, n=30)', marker='s')
    plt.scatter(x, y4, label='y4 (batchMax, n=30, ω=3)', marker='d')

    # Add titles and labels
    # plt.title("Scatter Plot of Data")
    plt.xlabel("Input scale (m)")
    plt.ylabel("Execution time [s]")
    plt.xscale("log")  # Optional, to set the x-axis to a logarithmic scale

    # Add a legend
    plt.legend()

    # Show the plot
    plt.show()

else:
    x = [100, 1000, 10000, 100000, 1000000]
    y1 = [487,2174,19049,187799,1875299] #(bitMax, n=15)
    y2 = [458,2145,19020,187770,1875270] #(batchMax, \omega=3)
    y3 = [989,4364,38114,375614,3750614] #(bitMax, n=30)
    y4 = [915,4290,38040,375540,3750540] #(batchMax, \omega=3)

    y1 = [e/(1024**2) for e in y1]
    y2 = [e/(1024**2) for e in y2]
    y3 = [e/(1024**2) for e in y3]
    y4 = [e/(1024**2) for e in y4]

    print(y1)
    print(y2)
    print(y3)
    print(y4)

    plt.figure(figsize=(5, 2.7), layout='constrained')

    # Create scatter plots
    plt.scatter(x, y1, label='y1 (bitMax, n=15)', marker='o')
    plt.scatter(x, y2, label='y2 (batchMax, n=15, ω=3)', marker='x')
    plt.scatter(x, y3, label='y3 (bitMax, n=30)', marker='s')
    plt.scatter(x, y4, label='y4 (batchMax, n=30, ω=3)', marker='d')

    # Add titles and labels
    # plt.title("Scatter Plot of Data")
    plt.xlabel("Input scale (m)")
    plt.ylabel("Communication per server[MB]")
    plt.xscale("log")  # Optional, to set the x-axis to a logarithmic scale

    # Add a legend
    plt.legend()

    # Show the plot
    plt.show()
