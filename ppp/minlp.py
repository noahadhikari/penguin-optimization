import math
from pathlib import Path
from instance import Instance
from ppp.point import Point
from solution import Solution

import pyomo.environ as pyo
from pyomo.opt import SolverFactory


def solve_minlp(instance: Instance) -> Solution:
  
  model = pyo.ConcreteModel()

  dim = instance.grid_side_length

  model.x = pyo.Set(initialize=range(dim))
  model.y = pyo.Set(initialize=range(dim))
  
  model.dimension = pyo.Param(default=instance.grid_side_length, within=pyo.PositiveIntegers)

  model.t = pyo.Var(model.x, model.y, within=pyo.Binary)

  model.CitiesSet = pyo.Set(initialize=instance.cities)

  model.cities = pyo.Param(model.CitiesSet)

  model.cover_cities = pyo.ConstraintList()

  for city in model.CitiesSet:
    model.cover_cities.add(
      sum(model.t[point.x, point.y] for point in Point.points_within_radius(city, instance.coverage_radius, dim))
      + model.t[city.x, city.y]
      >= 1
    )

  model.pprint()

  def objective_rule(model):
    w = [[] for _ in range(dim)]
    for x in range(dim):
      for y in range(dim):
        w_xy = 0
        for point in Point.points_within_radius(Point(x, y), instance.penalty_radius, dim):
          w_xy += model.t[point.x, point.y]
        w[x].append(w_xy)
    
    cost = 0.0

    for x in range(dim):
      for y in range(dim):
        # cost += 170.0 * model.t[x, y] * pyo.exp(0.17 * w[x][y]) 
        cost += 170.0 * model.t[x, y] * exp_expansion_1(w[x][y]) 

    model.dummy_constraint = pyo.Constraint(expr=pyo.summation(model.t) >= 0)

    return cost
  
  model.objective = pyo.Objective(rule=objective_rule, sense=pyo.minimize)
  
  # model.pprint()

  # solver = pyo.SolverFactory('ipopt')
  solver = pyo.SolverFactory('bonmin')
  solver.options['max_iter'] = 100
  results = solver.solve(model, tee=True)

  towers = []

  for x in range(dim):
    for y in range(dim):
      if model.t[x, y].value == 1:
        towers.append(Point(x, y))

  sol = Solution(
    instance=instance,
    towers=towers,
  )
  print(sol.valid())
  print(sol.penalty())
  return sol

def exp_expansion_1(w):
  w_0 = 6
  f0 = math.exp(0.17 * w_0)
  f1 = 0.17 * f0

  return f0 + f1 * (w - w_0)

def exp_expansion_2(w):
  w_0 = 6
  f0 = math.exp(0.17 * w_0)
  f1 = 0.17 * f0
  f2 = 0.17 * f1

  return f0 + f1 * (w - w_0) + 0.5 * f2 * (w - w_0)**2

if __name__ == "__main__":
  with Path("./inputs/small/001.in").open("r") as f:
    instance = Instance.parse(f.readlines())
    solve_minlp(instance)