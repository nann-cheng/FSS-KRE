import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator

# display execuation time figure or communication volume figure
is_show_time_figure=False

if is_show_time_figure:
    x = [100, 1000, 10000, 100000, 1000000]
    y1 = [2.93, 2.94, 3.03, 3.80, 8.89]
    
    y2 = [1.98, 2.01, 2.24, 3.76, 23.69]
    y3 = [8.50, 8.52, 8.69, 10.03, 21.40]
    y4 = [3.88, 3.92, 4.02, 7.66, 46.56]

    plt.figure(figsize=(5, 2.7), layout='constrained')

    # Create scatter plots
    plt.scatter(x, y1, label='y1 (bitKRE, n=15)', marker='o')
    plt.scatter(x, y2, label='y2 (batchKRE, ω=3)', marker='x')
    plt.scatter(x, y3, label='y3 (bitKRE, n=30)', marker='s')
    plt.scatter(x, y4, label='y4 (batchKRE, ω=3)', marker='d')

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
    y1 = [20183,21870,38745,207495,1894995] #(bitMax, n=15)
    y2 = [458,2145,19020,187770,1875270] #(batchMax, \omega=3)
    y3 = [989,4364,38114,375614,3750614] #(bitMax, n=30)
    y4 = [915,4290,38040,375540,3750540] #(batchMax, \omega=3)

    y1 = [e/(1024**2) for e in y1]
    y2 = [e/(1024**2) for e in y2]
    y3 = [e/(1024**2) for e in y3]
    y4 = [e/(1024**2) for e in y4]

    plt.figure(figsize=(5, 2.7), layout='constrained')

    # Create scatter plots
    plt.scatter(x, y1, label='y1 (bitbitKRE, n=15)', marker='o')
    plt.scatter(x, y2, label='y2 (batchbitKRE, ω=3)', marker='x')
    plt.scatter(x, y3, label='y3 (bitbitKRE, n=30)', marker='s')
    plt.scatter(x, y4, label='y4 (batchbitKRE, ω=3)', marker='d')

    # Add titles and labels
    # plt.title("Scatter Plot of Data")
    plt.xlabel("Input scale")
    plt.ylabel("Communication per server[KB]")
    plt.xscale("log")  # Optional, to set the x-axis to a logarithmic scale

    # Add a legend
    plt.legend()

    # Show the plot
    plt.show()
