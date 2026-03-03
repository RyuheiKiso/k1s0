package upstream

import (
	"net/http"
	"net/http/httputil"
	"net/url"
	"time"
)

// ReverseProxy wraps httputil.ReverseProxy and the parsed target URL.
type ReverseProxy struct {
	target *url.URL
	proxy  *httputil.ReverseProxy
}

func NewReverseProxy(upstreamURL string, timeout time.Duration) (*ReverseProxy, error) {
	target, err := url.Parse(upstreamURL)
	if err != nil {
		return nil, err
	}

	proxy := httputil.NewSingleHostReverseProxy(target)
	proxy.Transport = &http.Transport{
		ResponseHeaderTimeout: timeout,
	}

	return &ReverseProxy{
		target: target,
		proxy:  proxy,
	}, nil
}

func (r *ReverseProxy) ServeHTTP(w http.ResponseWriter, req *http.Request) {
	r.proxy.ServeHTTP(w, req)
}

func (r *ReverseProxy) Target() *url.URL {
	return r.target
}
