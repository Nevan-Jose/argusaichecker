'use client';
import { cn } from '@/lib/utils';
import { useTheme } from 'next-themes';
import React, { useEffect, useRef } from 'react';
import * as THREE from 'three';

type DottedSurfaceProps = Omit<React.ComponentProps<'div'>, 'ref'> & {
    /** THREE.js hex colour for the particles e.g. 0xffe0c2. Defaults to theme-based grey. */
    particleColor?: number;
    /** Whether to animate waves. Defaults to true. Pass false to show flat static dots. */
    playing?: boolean;
};

export function DottedSurface({ className, particleColor, playing = true, ...props }: DottedSurfaceProps) {
    const { theme } = useTheme();
    const containerRef = useRef<HTMLDivElement>(null);
    const playingRef   = useRef(playing);

    // Keep ref in sync with prop without recreating the scene
    useEffect(() => {
        playingRef.current = playing;
    }, [playing]);

    useEffect(() => {
        const el = containerRef.current;
        if (!el) return;

        const SEPARATION = 150;
        const AMOUNTX   = 40;
        const AMOUNTY   = 60;

        const scene  = new THREE.Scene();
        const camera = new THREE.PerspectiveCamera(60, window.innerWidth / window.innerHeight, 1, 10000);
        camera.position.set(0, 355, 1220);

        const renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true });
        renderer.setPixelRatio(window.devicePixelRatio);
        renderer.setSize(window.innerWidth, window.innerHeight);
        renderer.setClearColor(0x000000, 0);

        renderer.domElement.style.width  = '100%';
        renderer.domElement.style.height = '100%';
        renderer.domElement.style.display = 'block';

        el.appendChild(renderer.domElement);

        const geometry  = new THREE.BufferGeometry();
        const positions: number[] = [];
        for (let ix = 0; ix < AMOUNTX; ix++) {
            for (let iy = 0; iy < AMOUNTY; iy++) {
                positions.push(
                    ix * SEPARATION - (AMOUNTX * SEPARATION) / 2,
                    0,
                    iy * SEPARATION - (AMOUNTY * SEPARATION) / 2,
                );
            }
        }
        geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));

        const color    = particleColor ?? (theme === 'dark' ? 0xc8c8c8 : 0x444444);
        const material = new THREE.PointsMaterial({
            size: 8,
            color,
            transparent: true,
            opacity: 0.9,
            sizeAttenuation: true,
        });

        const points = new THREE.Points(geometry, material);
        scene.add(points);

        // Render the initial flat frame
        renderer.render(scene, camera);

        let count = 0;
        let rafId: number;

        const animate = () => {
            rafId = requestAnimationFrame(animate);

            if (playingRef.current) {
                const posAttr = geometry.attributes.position as THREE.BufferAttribute;
                const pos     = posAttr.array as Float32Array;
                let i = 0;
                for (let ix = 0; ix < AMOUNTX; ix++) {
                    for (let iy = 0; iy < AMOUNTY; iy++) {
                        pos[i * 3 + 1] =
                            Math.sin((ix + count) * 0.3) * 50 +
                            Math.sin((iy + count) * 0.5) * 50;
                        i++;
                    }
                }
                posAttr.needsUpdate = true;
                count += 0.1;
            }

            renderer.render(scene, camera);
        };

        const onResize = () => {
            camera.aspect = window.innerWidth / window.innerHeight;
            camera.updateProjectionMatrix();
            renderer.setSize(window.innerWidth, window.innerHeight);
        };

        window.addEventListener('resize', onResize);
        animate();

        return () => {
            cancelAnimationFrame(rafId);
            window.removeEventListener('resize', onResize);
            geometry.dispose();
            material.dispose();
            renderer.dispose();
            if (renderer.domElement.parentNode === el) {
                el.removeChild(renderer.domElement);
            }
        };
    }, [theme, particleColor]);

    return (
        <div
            ref={containerRef}
            className={cn('pointer-events-none', className)}
            {...props}
        />
    );
}
