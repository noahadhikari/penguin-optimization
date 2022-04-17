import random

NUM_CITIES = 52
DIMENSION = 50

coords = set()
while len(coords) < NUM_CITIES:
	coord = (random.randint(0, DIMENSION - 1), random.randint(0, DIMENSION - 1))
	coords.add(coord)

for coord in coords:
	print(f"{coord[0]} {coord[1]}")