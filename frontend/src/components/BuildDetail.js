import React, { useState, useEffect } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import axios from 'axios';
import {
  CheckCircle,
  XCircle,
  Clock,
  Bug,
  ShieldCheck,
  FileText,
  ArrowLeft,
  Warning,
  Wrench,
  Lightning,
  ClipboardText,
  Download,
  CaretDown,
  CaretUp
} from '@phosphor-icons/react';

const BACKEND_URL = process.env.REACT_APP_BACKEND_URL;
const API = `${BACKEND_URL}/api`;

export const BuildDetail = () => {
  const { buildId } = useParams();
  const navigate = useNavigate();
  const [build, setBuild] = useState(null);
  const [scanResults, setScanResults] = useState(null);
  const [complianceReport, setComplianceReport] = useState(null);
  const [sbom, setSbom] = useState(null);
  const [healthScore, setHealthScore] = useState(null);
  const [remediationSuggestions, setRemediationSuggestions] = useState(null);
  const [signature, setSignature] = useState(null);
  const [updateInfo, setUpdateInfo] = useState(null);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState('logs');
  
  // Vulnerability Remediation state
  const [vulnAnalysis, setVulnAnalysis] = useState(null);
  const [remediating, setRemediating] = useState(false);
  const [remediationResult, setRemediationResult] = useState(null);
  const [expandedCves, setExpandedCves] = useState({});
  const [copiedDockerfile, setCopiedDockerfile] = useState(false);

  useEffect(() => {
    fetchBuildDetails();
    const interval = setInterval(() => {
      if (build?.status && !['completed', 'failed'].includes(build.status)) {
        fetchBuildDetails();
      }
    }, 3000);

    return () => clearInterval(interval);
  }, [buildId]);

  const fetchBuildDetails = async () => {
    try {
      const buildRes = await axios.get(`${API}/builds/${buildId}`);
      setBuild(buildRes.data);

      if (buildRes.data.status === 'completed') {
        try {
          const promises = [
            axios.get(`${API}/builds/${buildId}/scan`),
            axios.get(`${API}/builds/${buildId}/compliance`),
            axios.get(`${API}/builds/${buildId}/sbom`),
            axios.get(`${API}/builds/${buildId}/health`),
            axios.get(`${API}/builds/${buildId}/remediation`),
            axios.get(`${API}/builds/${buildId}/check-updates`),
            axios.get(`${API}/builds/${buildId}/vulnerabilities/analysis`)
          ];
          
          // Add signature fetch if build is signed
          if (buildRes.data.is_signed) {
            promises.push(axios.get(`${API}/builds/${buildId}/signature`));
          }
          
          const responses = await Promise.all(promises);
          setScanResults(responses[0].data);
          setComplianceReport(responses[1].data);
          setSbom(responses[2].data);
          setHealthScore(responses[3].data);
          setRemediationSuggestions(responses[4].data);
          setUpdateInfo(responses[5].data);
          setVulnAnalysis(responses[6].data);
          
          if (buildRes.data.is_signed && responses[7]) {
            setSignature(responses[7].data);
          }
        } catch (err) {
          console.error('Error fetching additional data:', err);
        }
      }
    } catch (error) {
      console.error('Error fetching build details:', error);
    } finally {
      setLoading(false);
    }
  };

  const getStatusIcon = (status) => {
    switch (status) {
      case 'completed':
        return <CheckCircle size={32} weight="fill" className="text-[#34C759]" />;
      case 'failed':
        return <XCircle size={32} weight="fill" className="text-[#FF3B30]" />;
      default:
        return <Clock size={32} weight="fill" className="text-[#FFCC00] animate-pulse" />;
    }
  };

  const getVulnColor = (severity) => {
    const colors = {
      CRITICAL: 'text-[#FF3B30] bg-[#FF3B30]/10 border-[#FF3B30]/20',
      HIGH: 'text-[#FFCC00] bg-[#FFCC00]/10 border-[#FFCC00]/20',
      MEDIUM: 'text-[#002FA7] bg-[#002FA7]/10 border-[#002FA7]/20',
      LOW: 'text-[#4B5563] bg-black/5 border-black/10'
    };
    return colors[severity] || colors.LOW;
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center" data-testid="loading-build-detail">
        <div className="text-center">
          <Clock size={48} className="mx-auto mb-4 animate-pulse text-[#002FA7]" />
          <p className="text-sm uppercase tracking-wider text-[#4B5563]">Loading Build Details...</p>
        </div>
      </div>
    );
  }

  if (!build) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <XCircle size={48} className="mx-auto mb-4 text-[#FF3B30]" />
          <p className="text-[#4B5563] mb-4">Build not found</p>
          <Link to="/builds" className="btn-primary">Back to Builds</Link>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto px-6 sm:px-8 py-8" data-testid="build-detail-page">
      {/* Header */}
      <div className="mb-6">
        <button
          onClick={() => navigate('/builds')}
          className="flex items-center gap-2 text-sm text-[#4B5563] hover:text-[#002FA7] mb-4"
          data-testid="back-to-builds-btn"
        >
          <ArrowLeft size={16} />
          Back to Builds
        </button>
        
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-4">
            {getStatusIcon(build.status)}
            <div>
              <div className="flex items-center gap-3">
                <h2 className="text-4xl font-bold tracking-tighter" style={{fontFamily: 'Chivo'}}>{build.config_name}</h2>
                {build.is_signed && signature && (
                  <span className="text-xs px-2 py-1 bg-[#34C759]/10 text-[#34C759] border border-[#34C759]/20 rounded-sm font-medium" title="Image is cryptographically signed">
                    🔐 SIGNED
                  </span>
                )}
                {build.architecture && build.architecture.length > 1 && (
                  <span className="text-xs px-2 py-1 bg-[#002FA7]/10 text-[#002FA7] border border-[#002FA7]/20 rounded-sm font-medium">
                    MULTI-ARCH
                  </span>
                )}
              </div>
              <p className="text-sm text-[#4B5563] mt-1">{build.image_tag || 'Building...'}</p>
              {updateInfo && updateInfo.update_info.has_updates && (
                <div className="mt-2 flex items-center gap-2 text-xs">
                  <Warning size={14} className="text-[#FFCC00]" />
                  <span className="text-[#FFCC00]">Updates available - {updateInfo.recommendation.message}</span>
                </div>
              )}
            </div>
          </div>
          <div className="text-right">
            <div className="text-xs uppercase tracking-wider text-[#4B5563] mb-1">Status</div>
            <div className="text-lg font-bold uppercase" style={{fontFamily: 'Chivo'}}>{build.status}</div>
          </div>
        </div>
      </div>

      {/* Stats Cards */}
      {build.status === 'completed' && (
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8" data-testid="build-stats">
          {healthScore && (
            <div className="stat-card p-4">
              <div className="flex items-center gap-3 mb-2">
                <CheckCircle size={24} weight="bold" className="text-[#34C759]" />
                <span className="text-xs uppercase tracking-wider text-[#4B5563]">Health Score</span>
              </div>
              <div className="flex items-baseline gap-2">
                <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{healthScore.score}</div>
                <div className="text-2xl font-bold text-[#4B5563]" style={{fontFamily: 'Chivo'}}>/ 100</div>
              </div>
              <div className="mt-2 text-sm font-medium">Grade: {healthScore.grade} - {healthScore.status}</div>
            </div>
          )}
          
          <div className="stat-card p-4">
            <div className="flex items-center gap-3 mb-2">
              <ShieldCheck size={24} weight="bold" className="text-[#002FA7]" />
              <span className="text-xs uppercase tracking-wider text-[#4B5563]">Compliance</span>
            </div>
            <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{build.compliance_score}%</div>
          </div>

          <div className="stat-card p-4">
            <div className="flex items-center gap-3 mb-2">
              <Bug size={24} weight="bold" className="text-[#FF3B30]" />
              <span className="text-xs uppercase tracking-wider text-[#4B5563]">Critical Vulns</span>
            </div>
            <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>
              {build.vulnerability_count?.CRITICAL || 0}
            </div>
          </div>

          <div className="stat-card p-4">
            <div className="flex items-center gap-3 mb-2">
              <Bug size={24} weight="bold" className="text-[#FFCC00]" />
              <span className="text-xs uppercase tracking-wider text-[#4B5563]">High Vulns</span>
            </div>
            <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>
              {build.vulnerability_count?.HIGH || 0}
            </div>
          </div>
        </div>
      )}

      {/* Tabs */}
      <div className="bg-white border border-black/10 rounded-sm overflow-hidden">
        <div className="border-b border-black/10 flex overflow-x-auto">
          {['logs', 'vulnerabilities', 'compliance', 'remediation', 'sbom'].map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={`px-6 py-3 text-sm uppercase tracking-wider font-medium transition-colors whitespace-nowrap ${
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
          {/* Logs Tab */}
          {activeTab === 'logs' && (
            <div data-testid="logs-content">
              <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Build Logs</h3>
              <div className="bg-[#0A0A0A] text-[#34C759] p-4 rounded-sm font-mono text-sm overflow-x-auto">
                {build.logs && build.logs.length > 0 ? (
                  build.logs.map((log, idx) => (
                    <div key={idx} className="build-log" data-testid={`log-entry-${idx}`}>
                      {log}
                    </div>
                  ))
                ) : (
                  <div className="text-[#4B5563]">No logs available</div>
                )}
              </div>
            </div>
          )}

          {/* Vulnerabilities Tab - Enhanced with Remediation */}
          {activeTab === 'vulnerabilities' && (
            <div data-testid="vulnerabilities-content">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-xl font-bold" style={{fontFamily: 'Chivo'}}>Vulnerability Scan Results</h3>
                {vulnAnalysis && vulnAnalysis.analysis?.fixable_count > 0 && (
                  <button
                    onClick={async () => {
                      setRemediating(true);
                      try {
                        const res = await axios.post(`${API}/builds/${buildId}/remediate`);
                        setRemediationResult(res.data);
                      } catch (err) {
                        console.error('Remediation failed:', err);
                        alert('Remediation failed: ' + (err.response?.data?.detail || err.message));
                      } finally {
                        setRemediating(false);
                      }
                    }}
                    disabled={remediating}
                    className="flex items-center gap-2 px-4 py-2 bg-[#34C759] text-white rounded-sm hover:bg-[#34C759]/90 transition-colors disabled:opacity-50"
                    data-testid="auto-remediate-all-btn"
                  >
                    {remediating ? (
                      <>
                        <Clock size={16} className="animate-spin" />
                        Remediating...
                      </>
                    ) : (
                      <>
                        <Lightning size={16} weight="fill" />
                        Auto-Remediate All ({vulnAnalysis.analysis.fixable_count})
                      </>
                    )}
                  </button>
                )}
              </div>
              
              {/* Remediation Result Banner */}
              {remediationResult && (
                <div className="mb-6 p-4 bg-[#34C759]/10 border border-[#34C759]/30 rounded-sm" data-testid="remediation-result">
                  <div className="flex items-start justify-between mb-3">
                    <div className="flex items-center gap-2">
                      <CheckCircle size={24} weight="fill" className="text-[#34C759]" />
                      <h4 className="font-bold text-[#34C759]">Remediation Complete!</h4>
                    </div>
                    <button
                      onClick={() => setRemediationResult(null)}
                      className="text-[#4B5563] hover:text-black"
                    >
                      <XCircle size={20} />
                    </button>
                  </div>
                  
                  <div className="grid grid-cols-3 gap-4 mb-4">
                    <div className="text-center p-2 bg-white rounded-sm">
                      <div className="text-2xl font-bold text-[#34C759]">{remediationResult.fixes_count}</div>
                      <div className="text-xs uppercase tracking-wider text-[#4B5563]">Fixes Applied</div>
                    </div>
                    <div className="text-center p-2 bg-white rounded-sm">
                      <div className="text-2xl font-bold text-[#002FA7]">{remediationResult.delta_scan?.vulnerabilities_fixed || 0}</div>
                      <div className="text-xs uppercase tracking-wider text-[#4B5563]">CVEs Fixed</div>
                    </div>
                    <div className="text-center p-2 bg-white rounded-sm">
                      <div className="text-2xl font-bold">{remediationResult.delta_scan?.verification_passed ? 'PASS' : 'CHECK'}</div>
                      <div className="text-xs uppercase tracking-wider text-[#4B5563]">Verification</div>
                    </div>
                  </div>
                  
                  {/* Generated Dockerfile */}
                  <div className="mb-3">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm font-medium">Generated Remediated Dockerfile:</span>
                      <div className="flex gap-2">
                        <button
                          onClick={() => {
                            navigator.clipboard.writeText(remediationResult.dockerfile);
                            setCopiedDockerfile(true);
                            setTimeout(() => setCopiedDockerfile(false), 2000);
                          }}
                          className="flex items-center gap-1 px-2 py-1 text-xs bg-white border border-black/20 rounded-sm hover:border-[#002FA7]"
                          data-testid="copy-dockerfile-btn"
                        >
                          <ClipboardText size={14} />
                          {copiedDockerfile ? 'Copied!' : 'Copy'}
                        </button>
                        <button
                          onClick={() => {
                            const blob = new Blob([remediationResult.dockerfile], { type: 'text/plain' });
                            const url = URL.createObjectURL(blob);
                            const a = document.createElement('a');
                            a.href = url;
                            a.download = 'Dockerfile.remediated';
                            a.click();
                          }}
                          className="flex items-center gap-1 px-2 py-1 text-xs bg-white border border-black/20 rounded-sm hover:border-[#002FA7]"
                          data-testid="download-dockerfile-btn"
                        >
                          <Download size={14} />
                          Download
                        </button>
                      </div>
                    </div>
                    <pre className="bg-[#0A0A0A] text-[#34C759] p-3 rounded-sm text-xs font-mono overflow-x-auto max-h-60">
                      {remediationResult.dockerfile}
                    </pre>
                  </div>
                  
                  {/* Applied Fixes List */}
                  {remediationResult.applied_fixes?.length > 0 && (
                    <div>
                      <span className="text-sm font-medium mb-2 block">Applied Fixes:</span>
                      <div className="space-y-1">
                        {remediationResult.applied_fixes.map((fix, idx) => (
                          <div key={idx} className="flex items-center gap-2 text-sm bg-white px-2 py-1 rounded-sm">
                            <CheckCircle size={14} weight="fill" className="text-[#34C759]" />
                            <span className="font-mono text-xs">{fix.cve_id}</span>
                            <span className="text-[#4B5563]">-</span>
                            <span className="text-[#4B5563]">{fix.description}</span>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              )}
              
              {/* Remediation Summary */}
              {vulnAnalysis && (
                <div className="mb-6 p-4 border border-black/10 rounded-sm bg-black/5" data-testid="remediation-summary">
                  <h4 className="font-bold text-sm uppercase tracking-wider mb-3">Remediation Summary</h4>
                  <div className="grid grid-cols-4 gap-3">
                    <div className="text-center p-3 bg-white rounded-sm border border-black/10">
                      <div className="text-2xl font-bold">{vulnAnalysis.analysis?.total_vulnerabilities || 0}</div>
                      <div className="text-xs uppercase tracking-wider text-[#4B5563]">Total</div>
                    </div>
                    <div className="text-center p-3 bg-[#34C759]/10 rounded-sm border border-[#34C759]/20">
                      <div className="text-2xl font-bold text-[#34C759]">{vulnAnalysis.analysis?.fixable_count || 0}</div>
                      <div className="text-xs uppercase tracking-wider text-[#4B5563]">Auto-Fixable</div>
                    </div>
                    <div className="text-center p-3 bg-[#FFCC00]/10 rounded-sm border border-[#FFCC00]/20">
                      <div className="text-2xl font-bold text-[#FFCC00]">{vulnAnalysis.analysis?.patch_available_count || 0}</div>
                      <div className="text-xs uppercase tracking-wider text-[#4B5563]">Patch Available</div>
                    </div>
                    <div className="text-center p-3 bg-[#FF3B30]/10 rounded-sm border border-[#FF3B30]/20">
                      <div className="text-2xl font-bold text-[#FF3B30]">{vulnAnalysis.analysis?.manual_required_count || 0}</div>
                      <div className="text-xs uppercase tracking-wider text-[#4B5563]">Manual Required</div>
                    </div>
                  </div>
                  
                  {vulnAnalysis.analysis?.estimated_remediation_time && (
                    <div className="mt-3 text-sm text-[#4B5563]">
                      <Clock size={14} className="inline mr-1" />
                      Estimated remediation time: <strong>{vulnAnalysis.analysis.estimated_remediation_time}</strong>
                    </div>
                  )}
                </div>
              )}
              
              {scanResults ? (
                <div className="space-y-4">
                  {/* Severity Summary */}
                  <div className="grid grid-cols-4 gap-3 mb-6">
                    {Object.entries(scanResults.total_count).map(([severity, count]) => (
                      <div key={severity} className={`p-3 border rounded-sm ${getVulnColor(severity)}`}>
                        <div className="text-xs uppercase tracking-wider mb-1">{severity}</div>
                        <div className="text-2xl font-bold">{count}</div>
                      </div>
                    ))}
                  </div>

                  {/* Vulnerability List with Remediation Status */}
                  {vulnAnalysis?.analysis?.vulnerabilities_with_remediation ? (
                    <div className="space-y-2">
                      {['CRITICAL', 'HIGH', 'MEDIUM', 'LOW'].map(severity => {
                        const vulns = vulnAnalysis.analysis.vulnerabilities_with_remediation.filter(
                          v => scanResults.vulnerabilities[severity]?.some(sv => sv.id === v.id)
                        );
                        if (vulns.length === 0) return null;
                        
                        return (
                          <div key={severity} className="border border-black/10 rounded-sm p-4">
                            <h4 className="font-bold text-lg mb-3" style={{fontFamily: 'Chivo'}}>
                              {severity} ({vulns.length})
                            </h4>
                            <div className="space-y-2">
                              {vulns.map((vuln, idx) => (
                                <div 
                                  key={idx} 
                                  className={`border rounded-sm overflow-hidden ${
                                    vuln.auto_fixable 
                                      ? 'border-[#34C759]/30 bg-[#34C759]/5' 
                                      : vuln.fix_available 
                                        ? 'border-[#FFCC00]/30 bg-[#FFCC00]/5'
                                        : 'border-black/10 bg-black/5'
                                  }`}
                                  data-testid={`vuln-item-${vuln.id}`}
                                >
                                  <div 
                                    className="flex items-center justify-between p-3 cursor-pointer"
                                    onClick={() => setExpandedCves(prev => ({...prev, [vuln.id]: !prev[vuln.id]}))}
                                  >
                                    <div className="flex items-center gap-3">
                                      <Warning size={16} className={severity === 'CRITICAL' ? 'text-[#FF3B30]' : 'text-[#FFCC00]'} />
                                      <div>
                                        <div className="font-medium text-sm flex items-center gap-2">
                                          {vuln.id}
                                          {vuln.auto_fixable && (
                                            <span className="text-xs px-2 py-0.5 bg-[#34C759] text-white rounded-full font-medium">
                                              AUTO-FIXABLE
                                            </span>
                                          )}
                                          {!vuln.auto_fixable && vuln.fix_available && (
                                            <span className="text-xs px-2 py-0.5 bg-[#FFCC00] text-black rounded-full font-medium">
                                              PATCH AVAILABLE
                                            </span>
                                          )}
                                          {vuln.breaking_changes && (
                                            <span className="text-xs px-2 py-0.5 bg-[#FF3B30] text-white rounded-full font-medium">
                                              BREAKING
                                            </span>
                                          )}
                                        </div>
                                        <div className="text-xs text-[#4B5563]">{vuln.package}</div>
                                      </div>
                                    </div>
                                    <div className="flex items-center gap-2">
                                      {vuln.auto_fixable && (
                                        <button
                                          onClick={async (e) => {
                                            e.stopPropagation();
                                            try {
                                              const res = await axios.post(`${API}/builds/${buildId}/remediate/${vuln.id}`);
                                              alert(`Fix generated for ${vuln.id}!\n\n${res.data.fix_command || 'Check the fix details.'}`);
                                            } catch (err) {
                                              alert('Failed to generate fix: ' + (err.response?.data?.detail || err.message));
                                            }
                                          }}
                                          className="flex items-center gap-1 px-2 py-1 text-xs bg-[#34C759] text-white rounded-sm hover:bg-[#34C759]/90"
                                          data-testid={`fix-btn-${vuln.id}`}
                                        >
                                          <Wrench size={12} />
                                          Fix This
                                        </button>
                                      )}
                                      {expandedCves[vuln.id] ? <CaretUp size={16} /> : <CaretDown size={16} />}
                                    </div>
                                  </div>
                                  
                                  {expandedCves[vuln.id] && (
                                    <div className="px-3 pb-3 border-t border-black/10 pt-3">
                                      <div className="text-sm mb-2">{vuln.description}</div>
                                      {vuln.fixed_version && (
                                        <div className="text-xs text-[#4B5563] mb-2">
                                          <strong>Fixed in:</strong> {vuln.fixed_version}
                                        </div>
                                      )}
                                      {vuln.fix_command && (
                                        <div className="mt-2">
                                          <div className="text-xs font-medium mb-1">Fix Command:</div>
                                          <pre className="bg-[#0A0A0A] text-[#34C759] p-2 rounded-sm text-xs font-mono overflow-x-auto">
                                            {vuln.fix_command}
                                          </pre>
                                        </div>
                                      )}
                                    </div>
                                  )}
                                </div>
                              ))}
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  ) : (
                    // Fallback to original display
                    Object.entries(scanResults.vulnerabilities).map(([severity, vulns]) => (
                      vulns.length > 0 && (
                        <div key={severity} className="border border-black/10 rounded-sm p-4">
                          <h4 className="font-bold text-lg mb-3" style={{fontFamily: 'Chivo'}}>
                            {severity} ({vulns.length})
                          </h4>
                          <div className="space-y-2">
                            {vulns.map((vuln, idx) => (
                              <div key={idx} className="flex items-start gap-3 p-2 bg-black/5 rounded-sm">
                                <Warning size={16} className="mt-1" />
                                <div className="flex-1">
                                  <div className="font-medium text-sm">{vuln.id}</div>
                                  <div className="text-xs text-[#4B5563]">{vuln.package}</div>
                                  <div className="text-xs mt-1">{vuln.description}</div>
                                </div>
                              </div>
                            ))}
                          </div>
                        </div>
                      )
                    ))
                  )}
                </div>
              ) : (
                <div className="text-center py-8 text-[#4B5563]">
                  Scan results not available yet
                </div>
              )}
            </div>
          )}

          {/* Compliance Tab */}
          {activeTab === 'compliance' && (
            <div data-testid="compliance-content">
              <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Compliance Report</h3>
              {complianceReport ? (
                <div>
                  <div className="grid grid-cols-3 gap-3 mb-6">
                    <div className="p-3 border border-black/10 rounded-sm">
                      <div className="text-xs uppercase tracking-wider text-[#4B5563] mb-1">Passed</div>
                      <div className="text-2xl font-bold text-[#34C759]">{complianceReport.passed}</div>
                    </div>
                    <div className="p-3 border border-black/10 rounded-sm">
                      <div className="text-xs uppercase tracking-wider text-[#4B5563] mb-1">Warnings</div>
                      <div className="text-2xl font-bold text-[#FFCC00]">{complianceReport.warnings}</div>
                    </div>
                    <div className="p-3 border border-black/10 rounded-sm">
                      <div className="text-xs uppercase tracking-wider text-[#4B5563] mb-1">Failed</div>
                      <div className="text-2xl font-bold text-[#FF3B30]">{complianceReport.failed}</div>
                    </div>
                  </div>

                  <div className="space-y-2">
                    {complianceReport.checks.map((check, idx) => (
                      <div key={idx} className="border border-black/10 rounded-sm p-3 flex items-center justify-between">
                        <div className="flex items-center gap-3">
                          {check.status === 'passed' ? (
                            <CheckCircle size={20} weight="fill" className="text-[#34C759]" />
                          ) : (
                            <Warning size={20} weight="fill" className="text-[#FFCC00]" />
                          )}
                          <div>
                            <div className="font-medium text-sm">{check.description}</div>
                            <div className="text-xs text-[#4B5563] uppercase">{check.profile}</div>
                          </div>
                        </div>
                        <div className="text-xs uppercase tracking-wider font-bold">{check.status}</div>
                      </div>
                    ))}
                  </div>
                </div>
              ) : (
                <div className="text-center py-8 text-[#4B5563]">
                  Compliance report not available yet
                </div>
              )}
            </div>
          )}

          {/* Remediation Tab */}
          {activeTab === 'remediation' && (
            <div data-testid="remediation-content">
              <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Remediation Suggestions</h3>
              {remediationSuggestions ? (
                <div>
                  {/* CIS Benchmark Score */}
                  {remediationSuggestions.cis_benchmark && (
                    <div className="mb-6 p-4 border border-black/10 rounded-sm">
                      <h4 className="font-bold text-lg mb-3" style={{fontFamily: 'Chivo'}}>
                        CIS Benchmark Score: {remediationSuggestions.cis_benchmark.score}/100 (Grade: {remediationSuggestions.cis_benchmark.grade})
                      </h4>
                      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                        <div className="text-center p-2 bg-[#34C759]/5 rounded-sm">
                          <div className="text-xl font-bold text-[#34C759]">{remediationSuggestions.cis_benchmark.passed}</div>
                          <div className="text-xs uppercase tracking-wider text-[#4B5563]">Passed</div>
                        </div>
                        <div className="text-center p-2 bg-[#FF3B30]/5 rounded-sm">
                          <div className="text-xl font-bold text-[#FF3B30]">{remediationSuggestions.cis_benchmark.failed}</div>
                          <div className="text-xs uppercase tracking-wider text-[#4B5563]">Failed</div>
                        </div>
                        <div className="text-center p-2 bg-[#FFCC00]/5 rounded-sm">
                          <div className="text-xl font-bold text-[#FFCC00]">{remediationSuggestions.cis_benchmark.warnings}</div>
                          <div className="text-xs uppercase tracking-wider text-[#4B5563]">Warnings</div>
                        </div>
                        <div className="text-center p-2 bg-black/5 rounded-sm">
                          <div className="text-xl font-bold">{remediationSuggestions.cis_benchmark.total_checks}</div>
                          <div className="text-xs uppercase tracking-wider text-[#4B5563]">Total</div>
                        </div>
                      </div>
                    </div>
                  )}

                  {/* Remediation Suggestions */}
                  {remediationSuggestions.remediation_suggestions && remediationSuggestions.remediation_suggestions.length > 0 ? (
                    <div className="space-y-4">
                      <h4 className="font-bold text-lg mb-3" style={{fontFamily: 'Chivo'}}>Suggested Actions:</h4>
                      {remediationSuggestions.remediation_suggestions.map((suggestion, idx) => (
                        <div key={idx} className="border border-black/10 rounded-sm p-4">
                          <div className="flex items-start justify-between mb-2">
                            <h5 className="font-bold text-base" style={{fontFamily: 'Chivo'}}>{suggestion.title}</h5>
                            <div className="flex gap-2">
                              <span className={`text-xs px-2 py-1 rounded-sm font-medium ${
                                suggestion.severity === 'critical' ? 'bg-[#FF3B30]/10 text-[#FF3B30]' :
                                suggestion.severity === 'high' ? 'bg-[#FFCC00]/10 text-[#FFCC00]' :
                                'bg-[#002FA7]/10 text-[#002FA7]'
                              }`}>
                                {suggestion.severity.toUpperCase()}
                              </span>
                              <span className="text-xs px-2 py-1 bg-black/5 rounded-sm font-medium">
                                Effort: {suggestion.effort}
                              </span>
                            </div>
                          </div>
                          
                          <div className="mb-3 text-sm text-[#4B5563]">
                            <strong>Impact:</strong> {suggestion.impact}
                          </div>
                          
                          <div className="bg-[#0A0A0A] text-[#34C759] p-3 rounded-sm">
                            <pre className="text-xs font-mono whitespace-pre-wrap">{suggestion.remediation}</pre>
                          </div>
                          
                          <div className="mt-2 text-xs uppercase tracking-wider text-[#4B5563]">
                            Profile: {suggestion.profile}
                          </div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="text-center py-8 bg-[#34C759]/5 rounded-sm">
                      <CheckCircle size={48} className="mx-auto mb-3 text-[#34C759]" />
                      <p className="font-bold text-[#34C759]">No Remediations Needed!</p>
                      <p className="text-sm text-[#4B5563]">All compliance checks passed successfully.</p>
                    </div>
                  )}
                </div>
              ) : (
                <div className="text-center py-8 text-[#4B5563]">
                  Remediation suggestions not available yet
                </div>
              )}
            </div>
          )}

          {/* SBOM Tab */}
          {activeTab === 'sbom' && (
            <div data-testid="sbom-content">
              <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Software Bill of Materials</h3>
              {sbom ? (
                <div className="bg-[#0A0A0A] text-[#34C759] p-4 rounded-sm overflow-x-auto">
                  <pre className="text-xs font-mono">{JSON.stringify(sbom, null, 2)}</pre>
                </div>
              ) : (
                <div className="text-center py-8 text-[#4B5563]">
                  SBOM not available yet
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};