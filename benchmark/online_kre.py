import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator

# display execuation time figure or communication volume figure
is_show_time_figure=False

if is_show_time_figure:
    x = [100, 1000, 10000, 100000, 1000000]
    y1 = [2.93, 2.94, 3.03, 3.80, 8.89]#(bitKRE, n=15)
    y2 = [2.45, 2.48, 2.70, 4.30, 23.4]#(batchKRE, \omega=3)
    y3 = [5.76, 5.81, 5.97, 6.83, 17.8]#(bitKRE, n=30)
    y4 = [4.83, 4.87, 5.02, 8.60, 50.75]#(batchKRE, \omega=3)

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
    y1 = [20183,21870,38745,207495,1894995] #(bitKRE, n=15)
    y2 = [3513,5200,22075,190825,1878325] #(batchKRE, \omega=3)
    y3 = [40365,43740,77490,414990, 3789990] #(bitKRE, n=30)
    y4 = [7025,10400,44150,381650, 3756650] #(batchKRE, \omega=3)

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
