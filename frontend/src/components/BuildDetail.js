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
  Warning
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
            axios.get(`${API}/builds/${buildId}/check-updates`)
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
          
          if (buildRes.data.is_signed && responses[6]) {
            setSignature(responses[6].data);
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

          {/* Vulnerabilities Tab */}
          {activeTab === 'vulnerabilities' && (
            <div data-testid="vulnerabilities-content">
              <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Vulnerability Scan Results</h3>
              {scanResults ? (
                <div className="space-y-4">
                  <div className="grid grid-cols-4 gap-3 mb-6">
                    {Object.entries(scanResults.total_count).map(([severity, count]) => (
                      <div key={severity} className={`p-3 border rounded-sm ${getVulnColor(severity)}`}>
                        <div className="text-xs uppercase tracking-wider mb-1">{severity}</div>
                        <div className="text-2xl font-bold">{count}</div>
                      </div>
                    ))}
                  </div>

                  {Object.entries(scanResults.vulnerabilities).map(([severity, vulns]) => (
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
                  ))}
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