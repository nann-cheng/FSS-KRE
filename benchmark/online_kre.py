import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator

# display execuation time figure or communication volume figure
is_show_time_figure=True


# what's reported from trivalKre
# Amount [10,20,30,50]
# Time: [7.05,18.84,35.65,65.43]
# Commu: [1184,4364,9544,25904]

# around 5 seconds for bitwise kre
# [2.46,2.41,2.69,2.69]

if is_show_time_figure:
    x = [100, 1000, 10000, 100000, 1000000]
    y1 = [2.69, 2.61, 2.85, 3.36, 8.39]#(bitKRE, n=15)
    y2 = [2.34, 2.29, 2.38, 4.06, 16.71]#(batchKRE, \omega=3)

    y3 = [5.12, 4.95, 5.13, 6.42, 16.07]#(bitKRE, n=30)
    y4 = [4.43, 4.40, 4.67, 7.32, 34.56]#(batchKRE, \omega=3)

    plt.figure(figsize=(5, 2.7), layout='constrained')

    # Create scatter plots
    plt.scatter(x, y1, label='$\Pi_{\mathsf{Max1}} (n=15)$', marker='o')
    plt.scatter(x, y2, label='$\Pi_{\mathsf{Max2}} (n=15, \omega=3)$', marker='x')
    plt.scatter(x, y3, label='$\Pi_{\mathsf{Max1}} (n=30)$', marker='s')
    plt.scatter(x, y4, label='$\Pi_{\mathsf{Max2}} (n=30, \omega=3)$', marker='d')

    # Add titles and labels
    # plt.title("Scatter Plot of Data")
    plt.xlabel("Input scale (m)")
    plt.ylabel("Computation time [s]")
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
    plt.scatter(x, y1, label='$\Pi_{\mathsf{Max1}} (n=15)$', marker='o')
    plt.scatter(x, y2, label='$\Pi_{\mathsf{Max2}} (n=15, \omega=3)$', marker='x')
    plt.scatter(x, y3, label='$\Pi_{\mathsf{Max1}} (n=30)$', marker='s')
    plt.scatter(x, y4, label='$\Pi_{\mathsf{Max2}} (n=30, \omega=3)$', marker='d')

    # Add titles and labels
    # plt.title("Scatter Plot of Data")
    plt.xlabel("Input scale")
    plt.ylabel("Commu. volume [MB]")
    plt.xscale("log")  # Optional, to set the x-axis to a logarithmic scale

    # Add a legend
    plt.legend()

    # Show the plot
    plt.show()
