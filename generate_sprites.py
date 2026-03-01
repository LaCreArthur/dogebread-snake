#!/usr/bin/env python3
"""Generate pixel-art sprites for DogeBread Snake."""

from PIL import Image
import os
import math

os.makedirs("client/assets", exist_ok=True)

# Color palette
TRANS   = (0,   0,   0,   0)
FUR     = (210, 170, 95,  255)
FUR_D   = (165, 125, 55,  255)
FUR_L   = (235, 200, 130, 255)
EYE     = (25,  15,  8,   255)
SHINE   = (255, 255, 255, 255)
NOSE    = (90,  50,  20,  255)
MOUTH   = (80,  45,  15,  255)

def make_doge_head():
    img = Image.new("RGBA", (32, 32), TRANS)
    px = img.load()

    center = (16, 17)
    radius = 13

    # Face fill: roughly circular
    for y in range(32):
        for x in range(32):
            dx = x - center[0]
            dy = y - center[1]
            if (dx*dx + (dy*1.1)*(dy*1.1)) < radius*radius:
                px[x, y] = FUR

    # Ears (dark fur blobs at top)
    ear_points_l = [
        (7,3),(8,3),(9,3),
        (6,4),(7,4),(8,4),(9,4),(10,4),
        (6,5),(7,5),(8,5),(9,5),
        (5,6),(6,6),(7,6),
    ]
    ear_points_r = [(31-x, y) for x, y in ear_points_l]
    for x, y in ear_points_l + ear_points_r:
        if 0 <= x < 32 and 0 <= y < 32:
            px[x, y] = FUR_D

    # Side shadow
    for y in range(8, 28):
        for x in range(32):
            if px[x, y][3] > 0:
                if x < 10 or x > 22:
                    r,g,b,a = px[x,y]
                    px[x,y] = (int(r*0.88), int(g*0.88), int(b*0.88), a)

    # Forehead highlight
    for y in range(7, 13):
        for x in range(12, 20):
            if px[x, y][3] > 0:
                r,g,b,a = px[x,y]
                px[x,y] = (min(255, int(r*1.12)), min(255, int(g*1.1)), min(255, int(b*1.05)), a)

    # Eyes
    eye_centers = [(10, 15), (22, 15)]
    for (ex, ey) in eye_centers:
        for dy in range(-1, 2):
            for dx in range(-2, 3):
                nx, ny = ex+dx, ey+dy
                if 0<=nx<32 and 0<=ny<32:
                    if abs(dx) + abs(dy) <= 2:
                        px[nx, ny] = EYE
        px[ex+1, ey-1] = SHINE
        px[ex+2, ey-1] = SHINE
        px[ex+1, ey] = SHINE

    # Nose
    for x,y in [(15,19),(16,19),(17,19),(14,20),(15,20),(16,20),(17,20),(18,20),(14,21),(15,21),(16,21),(17,21),(18,21),(15,22),(16,22),(17,22)]:
        px[x, y] = NOSE

    # Mouth
    for x,y in [(13,24),(14,24),(17,24),(18,24),(14,25),(15,25),(16,25),(17,25)]:
        px[x, y] = MOUTH

    # Blush cheeks
    for (bx, by) in [(8, 19), (24, 19)]:
        for dy in range(-1, 2):
            for dx in range(-1, 2):
                nx, ny = bx+dx, by+dy
                if 0<=nx<32 and 0<=ny<32 and px[nx,ny][3] > 0:
                    r,g,b,a = px[nx,ny]
                    px[nx,ny] = (min(255,int(r*0.95+220*0.05)), min(255,int(g*0.9+100*0.1)), min(255,int(b*0.9+80*0.1)), a)

    return img


GOLD_L   = (255, 220, 80,  255)
GOLD     = (220, 175, 40,  255)
GOLD_D   = (160, 120, 20,  255)
GOLD_RIM = (100, 70,  10,  255)
WHITE_SH = (255, 245, 200, 255)

def make_coin():
    img = Image.new("RGBA", (32, 32), TRANS)
    px = img.load()

    center = (16, 16)
    r_outer = 13
    r_inner = 11

    for y in range(32):
        for x in range(32):
            dx = x - center[0]
            dy = y - center[1]
            dist = math.sqrt(dx*dx + dy*dy)
            if dist <= r_inner:
                grad = (dx + dy) / (r_inner * 2)
                r = int(GOLD[0] + (GOLD_L[0]-GOLD[0]) * (-grad * 0.5 + 0.25))
                g = int(GOLD[1] + (GOLD_L[1]-GOLD[1]) * (-grad * 0.5 + 0.25))
                b = int(GOLD[2] + (GOLD_L[2]-GOLD[2]) * (-grad * 0.5 + 0.25))
                px[x, y] = (max(0,min(255,r)), max(0,min(255,g)), max(0,min(255,b)), 255)
            elif dist <= r_outer:
                px[x, y] = GOLD_D
            elif dist <= r_outer + 1:
                px[x, y] = GOLD_RIM

    # D letter vertical bar
    for y in range(10, 24):
        if 0<=13<32: px[13, y] = GOLD_RIM
        if 0<=14<32: px[14, y] = GOLD_RIM

    # D horizontal bars
    for x in range(13, 21):
        if 0<=x<32: px[x, 10] = GOLD_RIM
        if 0<=x<32: px[x, 22] = GOLD_RIM

    # D right curve
    for angle_deg in range(-75, 76):
        angle = math.radians(angle_deg)
        cx = 14 + 6 * math.cos(angle)
        cy = 16 + 6 * math.sin(angle)
        nx, ny = int(round(cx)), int(round(cy))
        if 0<=nx<32 and 0<=ny<32:
            px[nx, ny] = GOLD_RIM

    # Dogecoin crossbar (Ð stroke)
    for x in range(11, 22):
        if 0<=x<32:
            px[x, 16] = GOLD_RIM
            px[x, 17] = GOLD_RIM

    # Shine highlight top-left
    for y in range(8, 14):
        for x in range(8, 15):
            dx_s = x - center[0]
            dy_s = y - center[1]
            if math.sqrt(dx_s*dx_s + dy_s*dy_s) <= r_inner:
                if px[x,y][3] > 0 and px[x,y] not in (GOLD_RIM, GOLD_D):
                    r,g,b,a = px[x,y]
                    px[x,y] = (min(255,r+30), min(255,g+20), min(255,b+10), a)

    px[10, 10] = WHITE_SH
    px[11, 10] = WHITE_SH
    px[10, 11] = WHITE_SH

    return img


if __name__ == "__main__":
    doge = make_doge_head()
    doge.save("client/assets/doge_head.png")
    print(f"Saved doge_head.png")

    coin = make_coin()
    coin.save("client/assets/coin.png")
    print(f"Saved coin.png")

    print("All sprites generated!")
