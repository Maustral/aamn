import React, { useEffect, useRef } from 'react';

interface Peer {
    node_id_hex: string;
    endpoint: string;
    latency_ms: number;
    reputation: number;
}

interface NetworkGraphProps {
    peers: Peer[];
    nodeId: string;
}

const NetworkGraph: React.FC<NetworkGraphProps> = ({ peers, nodeId }) => {
    const canvasRef = useRef<HTMLCanvasElement>(null);

    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas) return;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        let animationFrameId: number;
        const particles: any[] = [];
        const centerX = canvas.width / 2;
        const centerY = canvas.height / 2;

        // Local Node (Center)
        const localNode = {
            id: nodeId,
            x: centerX,
            y: centerY,
            radius: 12,
            color: '#00f2fe',
            isLocal: true,
        };

        const drawNode = (node: any) => {
            ctx.beginPath();
            ctx.arc(node.x, node.y, node.radius, 0, Math.PI * 2);
            ctx.fillStyle = node.color;
            ctx.shadowBlur = 15;
            ctx.shadowColor = node.color;
            ctx.fill();
            ctx.shadowBlur = 0;

            // Pulse effect
            ctx.beginPath();
            ctx.arc(node.x, node.y, node.radius + (Math.sin(Date.now() / 400) * 3 + 3), 0, Math.PI * 2);
            ctx.strokeStyle = node.color + '44';
            ctx.stroke();
        };

        const render = () => {
            ctx.clearRect(0, 0, canvas.width, canvas.height);

            const time = Date.now() / 1000;
            const radius = Math.min(canvas.width, canvas.height) * 0.35;

            // Draw Connection Lines based on peers
            peers.forEach((peer, i) => {
                const angle = (i / peers.length) * Math.PI * 2 + time * 0.1;
                const x = centerX + Math.cos(angle) * radius;
                const y = centerY + Math.sin(angle) * radius;

                // Draw Line to center
                ctx.beginPath();
                ctx.moveTo(centerX, centerY);
                ctx.lineTo(x, y);
                ctx.strokeStyle = `rgba(0, 242, 30.1, ${0.1 + peer.reputation * 0.2})`;
                ctx.lineWidth = 1;
                ctx.stroke();

                // Draw node
                ctx.beginPath();
                ctx.arc(x, y, 6, 0, Math.PI * 2);
                ctx.fillStyle = peer.latency_ms < 50 ? '#10b981' : '#f59e0b';
                ctx.fill();

                // Glow
                ctx.beginPath();
                ctx.arc(x, y, 10, 0, Math.PI * 2);
                ctx.fillStyle = (peer.latency_ms < 50 ? '#10b981' : '#f59e0b') + '33';
                ctx.fill();
            });

            drawNode(localNode);

            animationFrameId = requestAnimationFrame(render);
        };

        render();

        return () => {
            cancelAnimationFrame(animationFrameId);
        };
    }, [peers, nodeId]);

    return (
        <div className="glass-panel" style={{ padding: '0', overflow: 'hidden', height: '400px', position: 'relative' }}>
            <h3 style={{ position: 'absolute', top: '20px', left: '20px', margin: '0', fontSize: '1rem', color: 'white' }}>
                Live Mesh Topology
            </h3>
            <canvas
                ref={canvasRef}
                width={800}
                height={400}
                style={{ width: '100%', height: '100%' }}
            />
            <div style={{ position: 'absolute', bottom: '20px', right: '20px', fontSize: '0.7rem', color: 'var(--text-secondary)' }}>
                Interconnected Peers: {peers.length}
            </div>
        </div>
    );
};

export default NetworkGraph;
