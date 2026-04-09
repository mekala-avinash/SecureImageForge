import React, { useState, useEffect } from 'react';
import axios from 'axios';
import {
  Detective,
  Warning,
  CheckCircle,
  XCircle,
  Clock,
  ArrowsClockwise,
  Cube,
  Database,
  ShieldWarning,
  Eye
} from '@phosphor-icons/react';

const BACKEND_URL = process.env.REACT_APP_BACKEND_URL;
const API = `${BACKEND_URL}/api`;

export const DriftDetection = () => {
  const [scanResults, setScanResults] = useState(null);
  const [runtimeImages, setRuntimeImages] = useState([]);
  const [scanHistory, setScanHistory] = useState([]);
  const [stats, setStats] = useState(null);
  const [loading, setLoading] = useState(true);
  const [scanning, setScanning] = useState(false);
  const [activeTab, setActiveTab] = useState('dashboard');

  useEffect(() => {
    fetchData();
  }, []);

  const fetchData = async () => {
    try {
      const [imagesRes, historyRes, statsRes] = await Promise.all([
        axios.get(`${API}/drift/runtime-images`),
        axios.get(`${API}/drift/history`),
        axios.get(`${API}/drift/stats`)
      ]);
      
      setRuntimeImages(imagesRes.data.images);
      setScanHistory(historyRes.data.scans);
      setStats(statsRes.data);
    } catch (error) {
      console.error('Error fetching drift data:', error);
    } finally {
      setLoading(false);
    }
  };

  const runDriftScan = async () => {
    setScanning(true);
    try {
      const res = await axios.get(`${API}/drift/scan`);
      setScanResults(res.data);
      fetchData(); // Refresh history
    } catch (error) {
      alert('Drift scan failed: ' + (error.response?.data?.detail || error.message));
    } finally {
      setScanning(false);
    }
  };

  const getRiskColor = (level) => {
    switch (level) {
      case 'critical':
        return 'bg-[#FF3B30] text-white';
      case 'high':
        return 'bg-[#FFCC00] text-black';
      case 'medium':
        return 'bg-[#002FA7] text-white';
      case 'low':
        return 'bg-[#4B5563] text-white';
      default:
        return 'bg-[#34C759] text-white';
    }
  };

  const getDriftIcon = (hasDrift, riskLevel) => {
    if (!hasDrift) {
      return <CheckCircle size={20} weight="fill" className="text-[#34C759]" />;
    }
    if (riskLevel === 'critical') {
      return <XCircle size={20} weight="fill" className="text-[#FF3B30]" />;
    }
    return <Warning size={20} weight="fill" className="text-[#FFCC00]" />;
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center" data-testid="loading-drift">
        <div className="text-center">
          <Clock size={48} className="mx-auto mb-4 animate-pulse text-[#002FA7]" />
          <p className="text-sm uppercase tracking-wider text-[#4B5563]">Loading Drift Detection...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-6 sm:px-8 py-8" data-testid="drift-detection-page">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h2 className="text-4xl sm:text-5xl font-bold tracking-tighter mb-2" style={{fontFamily: 'Chivo'}}>
            Drift Detection
          </h2>
          <p className="text-base text-[#4B5563]">Monitor runtime containers for configuration drift from hardened templates</p>
        </div>
        <button
          onClick={runDriftScan}
          disabled={scanning}
          className="btn-primary flex items-center gap-2"
          data-testid="run-scan-btn"
        >
          {scanning ? (
            <>
              <ArrowsClockwise size={16} className="animate-spin" />
              Scanning...
            </>
          ) : (
            <>
              <Detective size={16} weight="bold" />
              Run Drift Scan
            </>
          )}
        </button>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-4 gap-4 mb-8" data-testid="drift-stats">
        <div className="stat-card p-4">
          <div className="flex items-center gap-3 mb-2">
            <Cube size={24} weight="bold" className="text-[#002FA7]" />
            <span className="text-xs uppercase tracking-wider text-[#4B5563]">Runtime Images</span>
          </div>
          <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{runtimeImages.length}</div>
        </div>
        <div className="stat-card p-4">
          <div className="flex items-center gap-3 mb-2">
            <Database size={24} weight="bold" className="text-[#4B5563]" />
            <span className="text-xs uppercase tracking-wider text-[#4B5563]">Clusters</span>
          </div>
          <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{stats?.monitored_clusters || 0}</div>
        </div>
        <div className="stat-card p-4">
          <div className="flex items-center gap-3 mb-2">
            <CheckCircle size={24} weight="bold" className="text-[#34C759]" />
            <span className="text-xs uppercase tracking-wider text-[#4B5563]">Compliant</span>
          </div>
          <div className="text-3xl font-bold text-[#34C759]" style={{fontFamily: 'Chivo'}}>
            {scanResults?.summary?.compliant || stats?.latest_scan?.compliant || '-'}
          </div>
        </div>
        <div className="stat-card p-4">
          <div className="flex items-center gap-3 mb-2">
            <ShieldWarning size={24} weight="bold" className="text-[#FF3B30]" />
            <span className="text-xs uppercase tracking-wider text-[#4B5563]">Drifted</span>
          </div>
          <div className="text-3xl font-bold text-[#FF3B30]" style={{fontFamily: 'Chivo'}}>
            {scanResults?.summary?.drifted || stats?.latest_scan?.drifted_count || '-'}
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="bg-white border border-black/10 rounded-sm overflow-hidden">
        <div className="border-b border-black/10 flex">
          {['dashboard', 'images', 'history'].map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={`px-6 py-3 text-sm uppercase tracking-wider font-medium transition-colors ${
                activeTab === tab
                  ? 'bg-[#002FA7] text-white'
                  : 'text-[#4B5563] hover:bg-black/5'
              }`}
              data-testid={`tab-${tab}`}
            >
              {tab}
            </button>
          ))}
        </div>

        <div className="p-6">
          {/* Dashboard Tab */}
          {activeTab === 'dashboard' && (
            <div data-testid="dashboard-content">
              <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Latest Scan Results</h3>
              
              {scanResults ? (
                <div>
                  {/* Summary */}
                  <div className="mb-6 p-4 bg-black/5 rounded-sm">
                    <div className="grid grid-cols-4 gap-4 text-center">
                      <div>
                        <div className="text-2xl font-bold">{scanResults.summary.total_images}</div>
                        <div className="text-xs uppercase tracking-wider text-[#4B5563]">Total Scanned</div>
                      </div>
                      <div>
                        <div className="text-2xl font-bold text-[#34C759]">{scanResults.summary.compliant}</div>
                        <div className="text-xs uppercase tracking-wider text-[#4B5563]">Compliant</div>
                      </div>
                      <div>
                        <div className="text-2xl font-bold text-[#FFCC00]">{scanResults.summary.drifted}</div>
                        <div className="text-xs uppercase tracking-wider text-[#4B5563]">Drifted</div>
                      </div>
                      <div>
                        <div className="text-2xl font-bold text-[#FF3B30]">{scanResults.summary.critical}</div>
                        <div className="text-xs uppercase tracking-wider text-[#4B5563]">Critical</div>
                      </div>
                    </div>
                    <div className="mt-3 text-xs text-center text-[#4B5563]">
                      Scanned at: {new Date(scanResults.scanned_at).toLocaleString()}
                    </div>
                  </div>

                  {/* Drift Results */}
                  <div className="space-y-3">
                    {scanResults.results.map((result, idx) => (
                      <div 
                        key={idx} 
                        className={`border rounded-sm p-4 ${
                          result.has_drift 
                            ? 'border-[#FF3B30]/30 bg-[#FF3B30]/5' 
                            : 'border-[#34C759]/30 bg-[#34C759]/5'
                        }`}
                        data-testid={`drift-result-${idx}`}
                      >
                        <div className="flex items-center justify-between">
                          <div className="flex items-center gap-3">
                            {getDriftIcon(result.has_drift, result.risk_level)}
                            <div>
                              <div className="font-medium">{result.image_tag || result.image_id}</div>
                              <div className="text-sm text-[#4B5563]">
                                {result.namespace} / {result.pod_name}
                              </div>
                            </div>
                          </div>
                          <div className="flex items-center gap-2">
                            {result.has_drift && (
                              <span className={`text-xs px-2 py-1 rounded-full ${getRiskColor(result.risk_level)}`}>
                                {result.risk_level?.toUpperCase() || 'UNKNOWN'}
                              </span>
                            )}
                            <span className="text-xs text-[#4B5563]">
                              {result.drift_count || 0} issue{result.drift_count !== 1 ? 's' : ''}
                            </span>
                          </div>
                        </div>

                        {result.has_drift && result.drift_details?.length > 0 && (
                          <div className="mt-3 pt-3 border-t border-black/10">
                            <div className="text-xs uppercase tracking-wider text-[#4B5563] mb-2">Drift Details:</div>
                            <div className="space-y-2">
                              {result.drift_details.map((detail, dIdx) => (
                                <div key={dIdx} className="flex items-start gap-2 text-sm bg-white/50 p-2 rounded-sm">
                                  <Warning size={14} className="mt-0.5 text-[#FF3B30]" />
                                  <div>
                                    <div className="font-medium">{detail.type}</div>
                                    <div className="text-xs text-[#4B5563]">{detail.message}</div>
                                  </div>
                                </div>
                              ))}
                            </div>
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              ) : (
                <div className="text-center py-12">
                  <Detective size={48} className="mx-auto mb-4 text-[#E5E7EB]" />
                  <p className="text-[#4B5563] mb-4">No scan results yet</p>
                  <button onClick={runDriftScan} className="btn-primary">
                    Run Your First Scan
                  </button>
                </div>
              )}
            </div>
          )}

          {/* Images Tab */}
          {activeTab === 'images' && (
            <div data-testid="images-content">
              <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Monitored Runtime Images</h3>
              
              {runtimeImages.length === 0 ? (
                <div className="text-center py-12">
                  <Cube size={48} className="mx-auto mb-4 text-[#E5E7EB]" />
                  <p className="text-[#4B5563]">No runtime images registered</p>
                </div>
              ) : (
                <div className="space-y-3">
                  {runtimeImages.map((image, idx) => (
                    <div key={idx} className="border border-black/10 rounded-sm p-4" data-testid={`image-${idx}`}>
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                          <Cube size={20} className="text-[#002FA7]" />
                          <div>
                            <div className="font-medium font-mono text-sm">{image.image_tag}</div>
                            <div className="text-xs text-[#4B5563]">
                              Namespace: {image.namespace} | Pod: {image.pod_name}
                            </div>
                          </div>
                        </div>
                        <div className="flex items-center gap-3 text-xs">
                          {image.has_shell && (
                            <span className="px-2 py-1 bg-[#FFCC00]/10 text-[#FFCC00] rounded-sm">HAS SHELL</span>
                          )}
                          {image.running_as_root && (
                            <span className="px-2 py-1 bg-[#FF3B30]/10 text-[#FF3B30] rounded-sm">ROOT USER</span>
                          )}
                          {!image.has_shell && !image.running_as_root && (
                            <span className="px-2 py-1 bg-[#34C759]/10 text-[#34C759] rounded-sm">HARDENED</span>
                          )}
                        </div>
                      </div>
                      <div className="mt-2 text-xs text-[#4B5563]">
                        Template: {image.template_id || 'Not linked'} | 
                        Last updated: {image.last_updated ? new Date(image.last_updated).toLocaleString() : 'Unknown'}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* History Tab */}
          {activeTab === 'history' && (
            <div data-testid="history-content">
              <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Scan History</h3>
              
              {scanHistory.length === 0 ? (
                <div className="text-center py-12">
                  <Clock size={48} className="mx-auto mb-4 text-[#E5E7EB]" />
                  <p className="text-[#4B5563]">No scan history available</p>
                </div>
              ) : (
                <div className="space-y-2">
                  {scanHistory.map((scan, idx) => (
                    <div key={idx} className="border border-black/10 rounded-sm p-4 flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <Eye size={20} className="text-[#4B5563]" />
                        <div>
                          <div className="font-medium text-sm">
                            Scan {scan.id?.substring(0, 8)}
                          </div>
                          <div className="text-xs text-[#4B5563]">
                            {new Date(scan.scanned_at).toLocaleString()}
                          </div>
                        </div>
                      </div>
                      <div className="flex items-center gap-4 text-sm">
                        <span>{scan.total_images} images</span>
                        <span className="text-[#FFCC00]">{scan.drifted_count} drifted</span>
                        <span className="text-[#FF3B30]">{scan.critical_drifts} critical</span>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
