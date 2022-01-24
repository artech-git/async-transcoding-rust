type JwtClaim struct {
	UserID int64
	jwt.StandardClaims
  }
  
  // GenerateAccessToken generates a jwt token and a refereshToken
  func (j *JwtWrapper) GenerateAccessToken(userID int64) (*AccessTokenResponse, rest_errors.RestError) {

	//intiallize the token response
	generatedToken := &AccessTokenResponse{}
	//intiallize the claim
	claims := &JwtClaim{
	  UserID: userID,
	  StandardClaims: jwt.StandardClaims{
		ExpiresAt: time.Now().Local().Add(time.Hour * time.Duration(j.ExpirationHours)).Unix(),
		Issuer:    j.Issuer,
	  },
	}
  
	//Create New Sign Method HS256 and assigned claim to the sign method
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
  
	//get the final accessToken token
	accessJWTToken, err := token.SignedString([]byte(j.SecretKey))
	if err != nil {
	  return nil, rest_errors.NewError(err.Error(), "generating_token_error", http.StatusNotAcceptable)
	}
	//assign the accessToken
	generatedToken.AccessJWTToken = accessJWTToken
  
	//create the refresh token
	//intiallize the claim
	refreshClaims := &JwtClaim{
	  UserID: userID,
	  StandardClaims: jwt.StandardClaims{
		ExpiresAt: time.Now().Local().Add(time.Hour * time.Duration(j.ExpirationHours*2)).Unix(),
		Issuer:    j.Issuer,
	  },
	}
  
	refreshToken := jwt.NewWithClaims(jwt.SigningMethodHS256, refreshClaims)
	refreshJWTToken, refreshErr := refreshToken.SignedString([]byte(j.RefreshKey))

	if refreshErr != nil {
	  return nil, rest_errors.NewError(refreshErr.Error(), "generating_token_error", http.StatusNotAcceptable)
	}
  
	//assign the refrestToken
	generatedToken.RefreshToken = refreshJWTToken
  
	//return final token
	return generatedToken, nil
  }

  //------------------------------------------------------------------------------------------------

  //ValidateToken validates the jwt token
func (j *JwtWrapper) ValidateToken(accessToken string, refreshToken string, isExpiredChecker bool) (map[string]*JwtClaim, rest_errors.RestError) {
	//parse the accessToken with claim added to it
	accesstoken, accessErr := jwt.ParseWithClaims(
	  accessToken,
	  &JwtClaim{},
	  func(accesstoken *jwt.Token) (interface{}, error) {
		return []byte(j.SecretKey), nil
	  },
	)
	if accessErr != nil {
	  return nil, rest_errors.NewError(fmt.Sprintf("token error %s", accessErr.Error()), "accesstoken_parsing_error", http.StatusUnauthorized)
	}
  
	//parse the refreshToken with claim added to it
	refreshtoken, refreshErr := jwt.ParseWithClaims(
	  refreshToken,
	  &JwtClaim{},
	  func(refreshtoken *jwt.Token) (interface{}, error) {
		return []byte(j.RefreshKey), nil
	  },
	)
	if refreshErr != nil {
	  return nil, rest_errors.NewError(fmt.Sprintf("token error %s", refreshErr.Error()), "refreshtoken_parsing_error", http.StatusUnauthorized)
	}
  
	//generate the accessToken claims
	accessTokenclaims, ok := accesstoken.Claims.(*JwtClaim)
	if !ok {
	  err := errors.New("couldn't parse accessToken claims")
	  return nil, rest_errors.NewError(err.Error(), "accesstokenclaim_parsing_error", http.StatusUnauthorized)
	}
  
	//generate the refreshToken claims
	refreshTokenclaims, ok := refreshtoken.Claims.(*JwtClaim)
	if !ok {
	  err := errors.New("couldn't parse refreshToken claims")
	  return nil, rest_errors.NewError(err.Error(), "refreshtokenclaim_parsing_error", http.StatusUnauthorized)
	}
  
	//check the accessToken claims is expired or not
	if !accessTokenclaims.IsExpired() {
	  err := errors.New("access token is not expired")
	  return nil, rest_errors.NewError(err.Error(), "token_not_expired", http.StatusBadRequest)
	}
  
	//check wether the refresh token is expired or not
	if refreshTokenclaims.IsExpired() {
	  err := errors.New("refresh token is expired")
	  return nil, rest_errors.NewError(err.Error(), "refreshtoken_expired", http.StatusBadRequest)
	}
  
	return map[string]*JwtClaim{
	  "accessTokenClaim":  accessTokenclaims,
	  "refreshTokenClaim": refreshTokenclaims,
	}, nil
  }
  
  //ValidateToken validates the jwt token
  func (j *JwtWrapper) ValidateRefreshToken(refreshToken string, isExpiredChecker bool) (map[string]*JwtClaim, rest_errors.RestError) {
  
	//parse the refreshToken with claim added to it
	refreshtoken, refreshErr := jwt.ParseWithClaims(
	  refreshToken,
	  &JwtClaim{},
	  func(refreshtoken *jwt.Token) (interface{}, error) {
		return []byte(j.RefreshKey), nil
	  },
	)
	if refreshErr != nil {
	  return nil, rest_errors.NewError(fmt.Sprintf("token error %s", refreshErr.Error()), "refreshtoken_parsing_error", http.StatusUnauthorized)
	}
  
	//generate the refreshToken claims
	refreshTokenclaims, ok := refreshtoken.Claims.(*JwtClaim)
	if !ok {
	  err := errors.New("couldn't parse refreshToken claims")
	  return nil, rest_errors.NewError(err.Error(), "refreshtokenclaim_parsing_error", http.StatusUnauthorized)
	}
  
	//check wether the refresh token is expired or not
	if refreshTokenclaims.IsExpired() {
	  return nil, rest_errors.TokenExpired()
	}
  
	return map[string]*JwtClaim{
	  "refreshTokenClaim": refreshTokenclaims,
	}, nil
  }
//=======================================================================================================

//parse the accessToken with claim added to it
accesstoken, accessErr := jwt.ParseWithClaims(

    accessToken,
    &JwtClaim{},    
	func(accesstoken *jwt.Token) (interface{}, error) {
      return []byte(j.SecretKey), nil
    },

  )