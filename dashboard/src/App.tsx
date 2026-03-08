import { useEffect, useState } from 'react';
import './index.css';
import NetworkGraph from './NetworkGraph';
import { Network, Activity, Globe, Shield, RefreshCw, StopCircle } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

interface NodeStatus {
  version: string;
  public_key_hex: string;
  active_circuits: number;
  connected_peers: number;
  bytes_sent: number;
  bytes_received: number;
  is_guard: boolean;
}

interface Peer {
  node_id_hex: string;
  endpoint: string;
  latency_ms: number;
  reputation: number;
}

function App() {
  const [status, setStatus] = useState<NodeStatus | null>(null);
  const [peers, setPeers] = useState<Peer[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actionMsg, setActionMsg] = useState('');
  const [token, setToken] = useState<string | null>(() => {
    return localStorage.getItem('aamn_token');
  });
  const [tokenInput, setTokenInput] = useState('');
  const [authError, setAuthError] = useState<string | null>(null);

  const fetchNodeState = async () => {
    if (!token) {
      return;
    }

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 1000);

      const headers: HeadersInit = {
        'Authorization': `Bearer ${token}`,
      };

      const [statusRes, peersRes] = await Promise.all([
        fetch("http://localhost:50052/api/status", { signal: controller.signal, headers }).then(r => {
          if (r.status === 401) {
            throw new Error('UNAUTHORIZED');
          }
          return r.json();
        }),
        fetch("http://localhost:50052/api/peers", { signal: controller.signal, headers }).then(r => {
          if (r.status === 401) {
            throw new Error('UNAUTHORIZED');
          }
          return r.json();
        })
      ]);

      clearTimeout(timeoutId);
      setStatus(statusRes);
      setPeers(peersRes);
      setError(null);
    } catch (e: any) {
      if (e.message === 'UNAUTHORIZED') {
        setAuthError("Invalid token. Please login again.");
        localStorage.removeItem('aamn_token');
        setToken(null);
      } else if (e.name === 'AbortError') {
        setError("Connection Timeout");
      } else {
        setError(e.message || "Failed to connect to AAMN node APIs");
      }
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (!token) return;
    fetchNodeState();
    const interval = setInterval(fetchNodeState, 2000); // Polling every 2s
    return () => clearInterval(interval);
  }, [token]);

  const handleGenerateNoise = async () => {
    if (!token) return;

    try {
      const res = await fetch("http://localhost:50052/api/noise", {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      }).then(r => {
        if (r.status === 401) {
          throw new Error('UNAUTHORIZED');
        }
        return r.json();
      });
      setActionMsg(res.message);
      setTimeout(() => setActionMsg(''), 4000);
    } catch (e: any) {
      if (e.message === 'UNAUTHORIZED') {
        setAuthError("Invalid token. Please login again.");
        localStorage.removeItem('aamn_token');
        setToken(null);
      } else {
        setActionMsg("Error: " + e.message);
      }
    }
  };

  const handleStopNode = async () => {
    if (window.confirm("Are you sure you want to shut down the node?")) {
      try {
        if (!token) return;
        const res = await fetch("http://localhost:50052/api/stop", {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${token}`,
          },
        }).then(r => {
          if (r.status === 401) {
            throw new Error('UNAUTHORIZED');
          }
          return r.json();
        });
        setActionMsg(res.message);
      } catch (e: any) {
        if (e.message === 'UNAUTHORIZED') {
          setAuthError("Invalid token. Please login again.");
          localStorage.removeItem('aamn_token');
          setToken(null);
        } else {
          setActionMsg("Error: " + e.message);
        }
      }
    }
  };

  const handleLogin = () => {
    const trimmed = tokenInput.trim();
    if (!trimmed) {
      setAuthError("Token is required");
      return;
    }
    localStorage.setItem('aamn_token', trimmed);
    setToken(trimmed);
    setAuthError(null);
    setLoading(true);
  };

  if (!token) {
    return (
      <div className="app-container" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100vh' }}>
        <div className="glass-panel" style={{ textAlign: 'center', maxWidth: '420px', width: '100%' }}>
          <h1 style={{ marginBottom: '16px' }}>AAMN Dashboard Login</h1>
          <p style={{ fontSize: '0.85rem', color: 'var(--text-secondary)', marginBottom: '16px' }}>
            Enter the control token configured in the AAMN node (env <code>AAMN_CONTROL_TOKEN</code>).
          </p>
          <input
            type="password"
            placeholder="Control token"
            value={tokenInput}
            onChange={(e) => setTokenInput(e.target.value)}
            style={{ width: '100%', padding: '10px 12px', borderRadius: '6px', border: '1px solid rgba(255,255,255,0.1)', background: 'rgba(0,0,0,0.4)', color: 'white', marginBottom: '12px' }}
          />
          <button
            className="btn btn-primary"
            style={{ width: '100%', height: '44px' }}
            onClick={handleLogin}
          >
            Connect
          </button>
          {authError && (
            <div style={{ marginTop: '12px', color: '#ff6b6b', fontSize: '0.8rem' }}>
              {authError}
            </div>
          )}
        </div>
      </div>
    );
  }

  if (loading && !status) return (
    <div className="app-container" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100vh' }}>
      <div className="glass-panel" style={{ textAlign: 'center' }}>
        <RefreshCw size={48} className="animate-spin" style={{ color: 'var(--accent-color)', marginBottom: '16px' }} />
        <h1>Initializing AAMN Node...</h1>
      </div>
    </div>
  );

  return (
    <div className="app-container">
      {/* Background Animated Meshes */}
      <div className="background-glow"></div>
      <div className="background-glow"></div>

      <header className="header animate-in">
        <div>
          <h1>AAMN Dashboard <span style={{ fontSize: '0.8rem', opacity: 0.6, verticalAlign: 'middle' }}>v{status?.version}</span></h1>
          <p style={{ color: 'var(--text-secondary)' }}>Adaptive Anonymous Mesh Network - Private & Resilient Control</p>
        </div>

        <div style={{ display: 'flex', gap: '12px' }}>
          {error && <div className="badge badge-danger">Connection Lost</div>}
          {!error && <div className="badge badge-success">● Connected</div>}
        </div>
      </header>

      <AnimatePresence>
        {actionMsg && (
          <motion.div
            initial={{ opacity: 0, y: -20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95 }}
            className="glass-panel"
            style={{ marginBottom: '20px', color: 'var(--accent-color)', border: '1px solid var(--accent-color)', padding: '12px 24px', fontWeight: 'bold' }}
          >
            {actionMsg}
          </motion.div>
        )}
      </AnimatePresence>

      <div className="grid animate-in" style={{ animationDelay: '0.1s' }}>
        <div className="glass-panel">
          <div className="stat-label"><Shield size={14} style={{ verticalAlign: 'middle', marginRight: '6px' }} /> Public Key (Node ID)</div>
          <div className="stat-value" style={{ fontSize: '0.9rem', wordBreak: 'break-all', marginTop: '16px', fontFamily: 'monospace', letterSpacing: '0.05em' }}>
            {status?.public_key_hex || "Loading..."}
          </div>
        </div>

        <div className="glass-panel">
          <div className="stat-label"><Network size={14} style={{ verticalAlign: 'middle', marginRight: '6px' }} /> Mesh Connections</div>
          <div className="stat-value">{status?.connected_peers || 0}</div>
        </div>

        <div className="glass-panel">
          <div className="stat-label"><Activity size={14} style={{ verticalAlign: 'middle', marginRight: '6px' }} /> Encrypted Traffic</div>
          <div className="stat-value">{status?.bytes_sent ? (Number(status.bytes_sent) / 1024).toFixed(2) : 0} KB</div>
        </div>

        <div className="glass-panel">
          <div className="stat-label"><Globe size={14} style={{ verticalAlign: 'middle', marginRight: '6px' }} /> Node Role</div>
          <div className="stat-value" style={{ color: status?.is_guard ? 'var(--success-color)' : 'var(--accent-color)' }}>
            {status?.is_guard ? "Guard Node" : "Mesh Node"}
          </div>
        </div>
      </div>

      <div className="grid animate-in" style={{ animationDelay: '0.2s', gridTemplateColumns: '1fr 340px' }}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
          <NetworkGraph peers={peers} nodeId={status?.public_key_hex || ''} />

          <div className="glass-panel">
            <h2 style={{ fontSize: '1.2rem', color: 'white', marginBottom: '20px' }}>Active Mesh Routing Table</h2>
            <div className="table-container">
              <table>
                <thead>
                  <tr>
                    <th>Node ID</th>
                    <th>Endpoint</th>
                    <th>Latency</th>
                    <th>Reputation</th>
                  </tr>
                </thead>
                <tbody>
                  {peers.length === 0 ? (
                    <tr><td colSpan={4} style={{ textAlign: 'center', padding: '40px', color: 'var(--text-secondary)' }}>Searching for peers in the DHT network...</td></tr>
                  ) : (
                    peers.map((peer, i) => (
                      <tr key={i}>
                        <td style={{ fontFamily: 'monospace', color: 'var(--accent-color)' }}>{peer.node_id_hex.substring(0, 16)}...</td>
                        <td>{peer.endpoint}</td>
                        <td><span className={peer.latency_ms < 60 ? "badge badge-success" : "badge badge-warning"}>{peer.latency_ms} ms</span></td>
                        <td>
                          <div style={{ width: '100%', backgroundColor: 'rgba(255,255,255,0.05)', borderRadius: '4px', height: '6px', marginTop: '4px' }}>
                            <div style={{ width: `${peer.reputation * 100}%`, height: '100%', backgroundColor: 'var(--accent-color)', borderRadius: '4px' }}></div>
                          </div>
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
          <div className="glass-panel" style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            <h2 style={{ fontSize: '1.2rem', color: 'white' }}>Security Commands</h2>
            <p style={{ fontSize: '0.85rem', color: 'var(--text-secondary)' }}>Execute low-level network operations directly on the AAMN core daemon.</p>

            <button
              className="btn btn-primary"
              onClick={handleGenerateNoise}
              style={{ height: '50px' }}
            >
              <RefreshCw size={18} style={{ marginRight: '8px' }} /> Force Chaff Injection
            </button>

            <button
              className="btn"
              style={{ borderColor: 'rgba(255,100,100,0.3)', color: '#ff6b6b', background: 'rgba(255,100,100,0.05)', height: '50px' }}
              onClick={handleStopNode}
            >
              <StopCircle size={18} style={{ marginRight: '8px' }} /> Emergency Stop
            </button>
          </div>

          <div className="glass-panel" style={{ flexGrow: 1 }}>
            <h2 style={{ fontSize: '1.2rem', color: 'white', marginBottom: '16px' }}>System Status</h2>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ fontSize: '0.85rem', color: 'var(--text-secondary)' }}>Circuit Rotation</span>
                <span className="badge badge-info">Every 10m</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ fontSize: '0.85rem', color: 'var(--text-secondary)' }}>Traffic Padding</span>
                <span className="badge badge-success">Enabled</span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ fontSize: '0.85rem', color: 'var(--text-secondary)' }}>Onion Layers</span>
                <span className="badge badge-info">3-5 Hops</span>
              </div>
            </div>

            <div style={{ marginTop: '40px', padding: '16px', background: 'rgba(0,0,0,0.2)', borderRadius: '8px', borderLeft: '3px solid var(--accent-color)' }}>
              <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', fontStyle: 'italic' }}>
                "Routing all local TCP traffic anonymously through the AAMN mesh via the integrated SOCKS5 gateway."
              </p>
            </div>
          </div>
        </div>
      </div>

      <footer style={{ marginTop: 'auto', padding: '40px 0', textAlign: 'center', color: 'var(--text-secondary)', fontSize: '0.8rem' }}>
        AAMN Adaptive Anonymous Mesh Network - Experimental Production v0.4
      </footer>
    </div>
  );
}

export default App;
