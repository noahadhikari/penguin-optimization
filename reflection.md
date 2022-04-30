# Reflection

### Describe the algorithm you used to generate your outputs. Why do you think it is a good approach?

We used a randomized hillclimbing algorithm to generate most of our outputs. The algorithm begins by calling an external ILP solver with a random seed to construct a valid tower solution to the input cities. The variables are all boolean: one for every (x, y) point a tower could possibly be at, corresponding to whether or not a tower gets placed there. A constraint for every city (i, j) is imposed such that the tower values within the service radius of that city sum to >= 1, corresponding to every city being covered by at least one tower. Finally, we minimize the number of towers.

Then, the hillclimbing part takes place - repeatedly, the algorithm checks if any of the towers can be removed and still result in a valid grid, then carry forth that removal. If no tower can be removed, the algorithm tries to move towers to different locations to find a less-penalized solution, until a (local) optimum is reached. It repeats the entire process until it is the same or better than the rank-0 position on the leaderboard, or for the specified number of iterations.

We noted that the scores attained by this algorithm were quite competitive, especially for noisier inputs. However, for some inputs (e.g. small/141), a solution with 5 towers is worse than one with 6 towers (this can be checked by eye). Additionally, for some of the very simple inputs (on large), the LP struggled to generate a minimum tower solution, and the algorithm failed to find a good solution. For these, because they were very simple inputs, we were able to match the top leaderboard score by constructing the output by hand.

### What other approaches did you try? How did they perform?

Our first approach was to reduce the problem to an LP and, additionally to the variables and constraints described above, impose penalty variables corresponding to the number of overlapping penalized towers for each point. Then, we would minimize this penalty. While this did work somewhat well, it was incredibly slow (only feasible for small inputs), and we found that simply randomizing many times (with no hillclimbing yet) worked just as well and was much faster (we called this approach RLP). 

Then, we noticed that there may be easy improvement within the LP that we hadn't considered, and turned to the hillclimbing algorithm described above. This algorithm tends to get stuck in local (not global) optima, which led us to repeat, leading to the final algorithm. 

# TODO (Daniel / Meshan) : Write about the greedy approaches we tried

# TODO (Daniel / Meshan) : What computational resources did you use? (e.g. AWS, instructional machines, etc.)
